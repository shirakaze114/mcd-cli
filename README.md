# mcd-cli

麦当劳 MCP CLI —— 基于麦当劳中国官方 MCP Server 的命令行点餐工具。

## 功能

- 🔍 查询附近门店、浏览菜单、查看餐品详情
- 🎫 查看/领取优惠券
- 🛒 计算价格、创建订单、查询订单状态
- 🎁 积分商城兑换
- 📅 活动日历、账户积分查询

## 安装

```bash
git clone <repo>
cd mcd-cli
cargo build --release
```

编译完成后，二进制文件位于 `target/release/mcd-cli`。

## 配置

### 1. 获取 MCP Token

访问 [https://open.mcd.cn/mcp](https://open.mcd.cn/mcp)，登录后进入控制台激活 MCP Token。

### 2. 保存 Token

```bash
# 保存到配置文件（~/.config/mcd-cli/config.toml）
./mcd-cli login --token <YOUR_MCP_TOKEN>

# 查看配置
./mcd-cli config
```

Token 优先级：命令行参数 `--token` > 环境变量 `MCD_MCP_TOKEN` > 配置文件。

## 使用

### 交互模式（推荐）

```bash
./mcd-cli
```

进入交互式菜单，按提示操作即可。

### 命令行模式

```bash
# 测试连接
./mcd-cli init

# 查询附近门店
./mcd-cli nearby --city "南京市" --keyword "南京审计大学" --be-type 1 --search-type 2

# 浏览菜单（到店取餐）
./mcd-cli menu --store 1990366 --order-type 1

# 浏览菜单（外送）
./mcd-cli menu --store 1960282 --be 196028202 --order-type 2

# 计算价格（到店取餐）
./mcd-cli price --store 1990366 --order-type 1 \
  --items '[{"productCode":"9900005462","quantity":1}]'

# 创建订单（到店取餐，需传入 takeWayCode）
./mcd-cli order create --store 1990366 --order-type 1 \
  --items '[{"productCode":"9900005462","quantity":1}]' \
  --take-way take-in-store

# 创建订单（外送）
./mcd-cli order create --store 1960282 --be 196028202 --address <ADDRESS_ID> --order-type 2 \
  --items '[{"productCode":"903050","quantity":1}]'

# 查询订单
./mcd-cli order query <ORDER_ID>

# 查看优惠券
./mcd-cli coupon my
./mcd-cli coupon receive

# 积分商城
./mcd-cli mall products
```

## 命令列表

| 命令 | 说明 |
|------|------|
| `init` | 测试 MCP 连接 |
| `login` | 保存 Token 到配置文件 |
| `config` | 查看当前配置 |
| `time` | 查看当前时间 |
| `calendar` | 查看活动日历 |
| `account` | 查看账户/积分 |
| `nearby` | 查询附近门店 |
| `address list` | 查看配送地址 |
| `address add` | 新增配送地址 |
| `menu` | 浏览菜单 |
| `detail` | 餐品详情 |
| `coupon store` | 门店可用优惠券 |
| `coupon my` | 我的优惠券 |
| `coupon available` | 可领优惠券列表 |
| `coupon receive` | 一键领券 |
| `mall products` | 积分兑换商品列表 |
| `mall detail` | 积分商品详情 |
| `mall exchange` | 积分兑换下单 |
| `price` | 计算价格 |
| `order create` | 创建订单 |
| `order query` | 查询订单 |
| `interactive` | 交互式菜单 |

## 技术说明

- 协议：MCP Streamable HTTP (`https://mcp.mcd.cn`)
- 认证：Bearer Token
- MCP 版本：`2025-06-18`

## 免责声明

本工具仅供学习交流使用，请遵守麦当劳相关服务条款。使用本工具产生的任何后果由使用者自行承担。
