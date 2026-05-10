use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use std::str::FromStr;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const CONFIG_SEED: &[u8] = b"config";
const VAULT_AUTH_SEED: &[u8] = b"vault-authority";
const PRODUCT_SEED: &[u8] = b"product";
const PURCHASE_RECEIPT_SEED: &[u8] = b"purchase-receipt";

const CGD_MINT: &str = "EcUKd3gxekBeJFwoFrzLmiSYFpUK9RSokn4ob21jwfur";
const OWNER_WALLET: &str = "9iu9zspt5gZkJKtW7PK2DvhoLV17dYvLRbAJxxNtpCcX";

#[program]
pub mod cgd_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let accepted_mint = Pubkey::from_str(CGD_MINT).map_err(|_| error!(CgdError::InvalidMint))?;
        let owner_wallet =
            Pubkey::from_str(OWNER_WALLET).map_err(|_| error!(CgdError::InvalidOwnerWallet))?;

        require_keys_eq!(
            ctx.accounts.authority.key(),
            owner_wallet,
            CgdError::Unauthorized
        );
        require_keys_eq!(
            ctx.accounts.accepted_mint.key(),
            accepted_mint,
            CgdError::InvalidMint
        );

        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.accepted_mint = ctx.accounts.accepted_mint.key();
        config.vault = ctx.accounts.vault.key();
        config.bump = ctx.bumps.config;
        config.vault_authority_bump = ctx.bumps.vault_authority;
        Ok(())
    }

    pub fn upsert_product(
        ctx: Context<UpsertProduct>,
        product_id: u64,
        price: u64,
        active: bool,
        metadata_uri: String,
    ) -> Result<()> {
        require!(metadata_uri.len() <= Product::MAX_URI_LEN, CgdError::UriTooLong);

        let product = &mut ctx.accounts.product;
        product.product_id = product_id;
        product.price = price;
        product.active = active;
        product.metadata_uri = metadata_uri;
        product.bump = ctx.bumps.product;
        Ok(())
    }

    pub fn purchase_product(
        ctx: Context<PurchaseProduct>,
        order_id: u64,
        quantity: u64,
    ) -> Result<()> {
        require!(quantity > 0, CgdError::InvalidQuantity);

        let product = &ctx.accounts.product;
        require!(product.active, CgdError::ProductInactive);

        let total_price = product
            .price
            .checked_mul(quantity)
            .ok_or(error!(CgdError::MathOverflow))?;

        let receipt = &mut ctx.accounts.purchase_receipt;
        receipt.order_id = order_id;
        receipt.buyer = ctx.accounts.buyer.key();
        receipt.product = product.key();
        receipt.quantity = quantity;
        receipt.total_paid = total_price;
        receipt.created_at = Clock::get()?.unix_timestamp;
        receipt.bump = ctx.bumps.purchase_receipt;

        let transfer_accounts = Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let transfer_ctx =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts);
        token::transfer(transfer_ctx, total_price)?;

        Ok(())
    }

    pub fn sync_wallet_balance(ctx: Context<SyncWalletBalance>, target_balance: u64) -> Result<()> {
        let current_wallet_balance = ctx.accounts.user_token_account.amount;

        require!(
            target_balance >= current_wallet_balance,
            CgdError::TargetBelowCurrentBalance
        );

        let delta = target_balance
            .checked_sub(current_wallet_balance)
            .ok_or(error!(CgdError::MathOverflow))?;

        if delta == 0 {
            return Ok(());
        }

        require!(ctx.accounts.vault.amount >= delta, CgdError::InsufficientVaultBalance);

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTH_SEED, &[ctx.accounts.config.vault_authority_bump]]];
        let transfer_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
        token::transfer(transfer_ctx, delta)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub accepted_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + StoreConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, StoreConfig>,
    /// CHECK: PDA used only as token vault authority.
    #[account(
        seeds = [VAULT_AUTH_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = accepted_mint,
        associated_token::authority = vault_authority
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(product_id: u64)]
pub struct UpsertProduct<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority @ CgdError::Unauthorized
    )]
    pub config: Account<'info, StoreConfig>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Product::INIT_SPACE,
        seeds = [PRODUCT_SEED, &product_id.to_le_bytes()],
        bump
    )]
    pub product: Account<'info, Product>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(order_id: u64)]
pub struct PurchaseProduct<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, StoreConfig>,
    #[account(
        seeds = [PRODUCT_SEED, &product.product_id.to_le_bytes()],
        bump = product.bump
    )]
    pub product: Account<'info, Product>,
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key() @ CgdError::InvalidBuyerTokenAccount,
        constraint = buyer_token_account.mint == config.accepted_mint @ CgdError::InvalidMint
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = vault.key() == config.vault @ CgdError::InvalidVault,
        constraint = vault.mint == config.accepted_mint @ CgdError::InvalidMint
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = buyer,
        space = 8 + PurchaseReceipt::INIT_SPACE,
        seeds = [PURCHASE_RECEIPT_SEED, buyer.key().as_ref(), &order_id.to_le_bytes()],
        bump
    )]
    pub purchase_receipt: Account<'info, PurchaseReceipt>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SyncWalletBalance<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority @ CgdError::Unauthorized
    )]
    pub config: Account<'info, StoreConfig>,
    /// CHECK: PDA used only as token vault authority.
    #[account(
        seeds = [VAULT_AUTH_SEED],
        bump = config.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = vault.key() == config.vault @ CgdError::InvalidVault,
        constraint = vault.mint == config.accepted_mint @ CgdError::InvalidMint
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = accepted_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    /// CHECK: Wallet receiving CGD.
    pub user: UncheckedAccount<'info>,
    #[account(address = config.accepted_mint @ CgdError::InvalidMint)]
    pub accepted_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct StoreConfig {
    pub authority: Pubkey,
    pub accepted_mint: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
    pub vault_authority_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Product {
    pub product_id: u64,
    pub price: u64,
    pub active: bool,
    #[max_len(200)]
    pub metadata_uri: String,
    pub bump: u8,
}

impl Product {
    pub const MAX_URI_LEN: usize = 200;
}

#[account]
#[derive(InitSpace)]
pub struct PurchaseReceipt {
    pub order_id: u64,
    pub buyer: Pubkey,
    pub product: Pubkey,
    pub quantity: u64,
    pub total_paid: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[error_code]
pub enum CgdError {
    #[msg("Only the configured authority can perform this action.")]
    Unauthorized,
    #[msg("The provided mint does not match the configured CGD mint.")]
    InvalidMint,
    #[msg("The configured owner wallet is invalid.")]
    InvalidOwnerWallet,
    #[msg("The vault account is invalid.")]
    InvalidVault,
    #[msg("The buyer token account is invalid.")]
    InvalidBuyerTokenAccount,
    #[msg("Product is inactive.")]
    ProductInactive,
    #[msg("Quantity must be greater than zero.")]
    InvalidQuantity,
    #[msg("Arithmetic overflow occurred.")]
    MathOverflow,
    #[msg("Metadata URI is too long.")]
    UriTooLong,
    #[msg("Target balance cannot be below the current wallet balance.")]
    TargetBelowCurrentBalance,
    #[msg("Vault balance is insufficient.")]
    InsufficientVaultBalance,
}
