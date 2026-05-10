# Devnet Deployment Guide

这个项目目前已经具备合约代码和仓库结构，但要真正部署到 Solana devnet，还需要本地部署身份和标准 CLI 工具链。

## 当前状态

已完成：

- Anchor 合约代码
- GitHub 仓库同步
- Rust 工具链基础安装

未完成：

- 安装 `solana` CLI
- 安装 `anchor` CLI
- 本地部署钱包私钥文件
- 实际 `devnet` 部署

## 为什么 Phantom 不能直接代替部署私钥

前端用户调用 `purchase_product` 这类普通交易时，可以通过 Phantom 弹窗签名。

但 `anchor deploy` / `solana program deploy` 属于程序部署流程，标准方式要求：

- 本地可访问的部署钱包私钥
- 本地可访问的 program keypair
- CLI 在多笔部署交易中持续签名

所以单靠浏览器钱包弹窗，当前这个桌面工作流里不能可靠完成程序部署。

## 如果你坚持使用 Phantom 里的钱包

可以导出 Phantom 的 Solana 私钥，再在本地转换成 CLI 用的 `id.json` 文件。

Phantom 官方帮助中心说明了可以查看或导出私钥，入口通常是：

- `Settings`
- `Manage Accounts`
- 选择对应账户
- `Show Private Key`
- 选择 `Solana`

官方来源：

- [Phantom Help: View or export your recovery phrase or private keys in Phantom](https://help.phantom.com/hc/en-us/articles/25334064171795-View-or-export-your-recovery-phrase-or-private-keys-in-Phantom)

导出后，在本地项目目录运行：

```powershell
npm install
npm run import:phantom-key
```

然后：

1. 把 Phantom 导出的 Solana 私钥粘贴进本地终端
2. 看脚本显示的钱包地址是不是你要用的地址
3. 输入 `YES`
4. 脚本会生成 `C:\Users\Candy\.config\solana\id.json`

这一步只在你的电脑本地进行，不会上传私钥。

## 你需要准备的最少条件

### 1. 本地部署钱包私钥

把私钥文件放到：

`C:\Users\Candy\.config\solana\id.json`

这个钱包建议：

- 是 devnet 专用钱包
- 有足够的 devnet SOL
- 如果你要保持业务一致，最好就是你指定的管理员地址对应私钥

### 2. 安装工具

需要本机可用：

- Rust
- Solana CLI
- Anchor CLI

## 部署步骤

### 1. 生成或确认 Program ID

先生成程序 keypair，例如：

```powershell
solana-keygen new -o target\deploy\cgd_store-keypair.json
```

读取程序地址：

```powershell
solana address -k target\deploy\cgd_store-keypair.json
```

### 2. 回填 Program ID

把上一步得到的 Program ID 写回项目：

```powershell
node scripts/set-program-id.mjs <YOUR_PROGRAM_ID>
```

这个脚本会更新：

- `Anchor.toml`
- `programs/cgd_store/src/lib.rs`

### 3. 配置 devnet

```powershell
solana config set --url https://api.devnet.solana.com
solana config set --keypair C:\Users\Candy\.config\solana\id.json
```

### 4. 构建与部署

```powershell
anchor build
anchor deploy
```

### 5. 初始化程序

部署完成后，调用 `initialize()` 创建配置和程序金库 `vault`。

### 6. 给程序金库充值 CGD

从管理员钱包把一部分 `CGD` 转到 `vault`，供后续 `sync_wallet_balance()` 给用户补发金币。

## 部署完成后要更新仓库

完成部署后，建议提交以下变化到 GitHub：

- 真实 Program ID
- 如有新增脚本或前端配置，也一起提交

```powershell
git add .
git commit -m "Update deployed devnet program id"
git push
```
