# CGD Store Program

这个项目是一个基于 Solana devnet + Anchor 的 `CGD` 商城合约示例，适配你提供的代币：

- Token name: `Candy Golden Coins`
- Symbol: `CGD`
- Mint: `EcUKd3gxekBeJFwoFrzLmiSYFpUK9RSokn4ob21jwfur`
- 管理钱包: `9iu9zspt5gZkJKtW7PK2DvhoLV17dYvLRbAJxxNtpCcX`

## 这个合约解决什么问题

1. 用户点击商品时，前端拉起钱包签名，调用 `purchase_product`，从用户钱包里的 `CGD` 扣款。
2. 商家钱包先把一部分 `CGD` 充值到程序金库 `vault`。
3. 当前端累计金币增加时，后端或前端管理员调用 `sync_wallet_balance(target_balance)`，把用户钱包里的 `CGD` 补到目标数量。

## 重要边界

“自动同步到钱包”不能凭空发生，Solana 上任何余额变化都必须对应一笔链上交易。

所以推荐的业务流程是：

1. 你的业务系统记录用户累计金币，例如数据库中的 `target_balance`。
2. 当累计值变大时，服务端用管理员钱包调用 `sync_wallet_balance`。
3. 合约会读取用户当前 CGD 钱包余额，只补发差额 `delta` 到用户 ATA。

这个设计可以避免重复发币。

## 指令说明

### `initialize()`

初始化商城配置，并创建程序金库 `vault`：

- 仅允许钱包 `9iu9zspt5gZkJKtW7PK2DvhoLV17dYvLRbAJxxNtpCcX`
- 仅接受 mint `EcUKd3gxekBeJFwoFrzLmiSYFpUK9RSokn4ob21jwfur`

### `upsert_product(product_id, price, active, metadata_uri)`

创建或更新商品：

- `product_id`: 商品 ID
- `price`: 商品价格，单位是最小 token 单位
- `active`: 是否启用
- `metadata_uri`: 商品说明或前端元数据地址

### `purchase_product(order_id, quantity)`

用户购买商品：

- 用户钱包签名
- 从用户 CGD token account 转账到程序金库
- 写入一条 `purchase_receipt`

### `sync_wallet_balance(target_balance)`

把用户钱包中的 CGD 同步到目标累计值：

- 只有管理员可调用
- 如果用户当前余额是 `100`，目标是 `140`，则只补 `40`
- 如果目标小于当前余额，则拒绝执行

## 账户模型

- `StoreConfig`: 全局配置
- `Product`: 商品配置
- `PurchaseReceipt`: 订单收据
- `vault`: 程序金库，持有 CGD

## 部署前要做的事

### 1. 安装依赖

需要先安装：

- Rust
- Solana CLI
- Anchor CLI
- Node.js / Yarn

### 2. 修改程序 ID

当前示例使用的是 Anchor 默认示例 Program ID：

`Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`

正式部署前请执行：

```bash
anchor keys list
```

然后把以下两个位置改成你的真实 Program ID：

- `Anchor.toml`
- `programs/cgd_store/src/lib.rs`

### 3. 初始化金库

部署并执行 `initialize` 后，会创建程序金库 `vault`。  
你需要从钱包 `9iu9zspt5gZkJKtW7PK2DvhoLV17dYvLRbAJxxNtpCcX` 持有的 CGD 中，转一部分到 `vault`，作为后续同步奖励余额的资金来源。

## 前端接入建议

### 购买商品

前端流程：

1. 用户连接钱包
2. 查询用户的 CGD ATA
3. 调用 `purchase_product(order_id, quantity)`
4. 钱包弹窗确认

### 累计金币同步

推荐后端流程：

1. 业务系统计算用户累计金币 `target_balance`
2. 查询用户当前 CGD ATA 余额
3. 如果 `target_balance > current_balance`，调用 `sync_wallet_balance(target_balance)`

## 注意

你提供的这串信息：

`RdCqZFD185ZAqyqfyE251LV5VN5M5HU3mVRMMzeKTJc`

看起来像是某个 token account 地址，不是普通钱包地址。这个合约当前是按 “管理员钱包 + 程序金库” 的标准做法来写的，没有把这个 token account 硬编码进去，因为最安全的方式是让程序自己创建并控制金库。

如果你希望：

- 直接从某个既有 token account 扣/发 CGD
- 允许多个商户钱包
- 购买后自动记录订单状态、发货状态、退款
- 用 PDA 记录用户累计积分而不是直接以钱包余额作为累计值

我可以继续帮你补第二版。
