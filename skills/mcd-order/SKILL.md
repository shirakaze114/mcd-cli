# mcd-order

麦当劳 MCP 点餐助手 Skill。

## 描述

帮助用户通过麦当劳官方 MCP Server 完成浏览菜单、计算价格、下单点餐等操作。支持到店取餐、外送、得来速车道取餐、企业团餐四种场景，支持预约下单。

## 使用场景

- 用户想查看附近麦当劳门店（到店/得来速）
- 用户想浏览菜单并选择商品
- 用户想计算订单价格
- 用户想下单（到店取餐、外送、得来速、团餐）
- 用户想查询订单状态
- 用户想查看/领取优惠券
- 用户想查看积分或兑换积分商品（虚拟券/实物）
- 用户想查看餐品营养信息
- 用户想预约下单

## 前置条件

用户需要在 `https://open.mcd.cn/mcp` 申请并激活 MCP Token。Token 可以通过以下方式配置：

1. 命令行：`mcd-cli login --token <TOKEN>`（保存到配置文件）
2. 环境变量：`MCD_MCP_TOKEN=<TOKEN>`
3. 命令行参数：`mcd-cli --token <TOKEN> <command>`

## 工作流

### 1. 查询门店

**到店/得来速：**
```bash
mcd-cli nearby --city <城市> --keyword <关键词> --be-type 1 --search-type 2
```
- `be-type 1` = 到店自取
- `be-type 5` = 得来速车道取餐

**外送/团餐（需先查询地址）：**
```bash
mcd-cli address list --be-type 2
mcd-cli delivery-stores --address-id <ADDRESS_ID> --be-type 2
```
- `be-type 2` = 麦乐送
- `be-type 6` = 团餐

获取 `storeCode` 和 `storeName`。记录 `storeCode` 供后续使用。

### 2. 团餐助餐服务查询

```bash
mcd-cli catering --store <STORE_CODE> --be <BE_CODE>
```
获取 `gmServiceCode`（团餐下单必填）。

### 3. 浏览菜单

```bash
mcd-cli menu --store <STORE_CODE> --order-type 1 --be-type 1
```

从菜单中选择商品，记录 `productCode`。

**参数说明：**
- `order-type`: 1=到店取餐, 2=外送
- `be-type`: 1=到店取餐, 2=麦乐送, 5=得来速, 6=企业团餐
- `reservation-date`: 预约时间，格式 `yyyy-MM-dd HH:mm`

### 4. 计算价格

```bash
mcd-cli price --store <STORE_CODE> --order-type 1 --be-type 1 \
  --items '[{"productCode":"<CODE>","quantity":1}]'
```

从结果中获取 `takeWayList[].code`（到店/得来速创建订单时需要）。

**使用优惠券：**
```bash
mcd-cli price --store <STORE_CODE> --order-type 1 --be-type 1 \
  --items '[{"productCode":"<CODE>","quantity":1}]' \
  --coupon-id <COUPON_ID>
```

### 5. 创建订单

**到店取餐/得来速：**
```bash
mcd-cli order create --store <STORE_CODE> --order-type 1 --be-type 1 \
  --items '[{"productCode":"<CODE>","quantity":1}]' \
  --take-way <TAKE_WAY_CODE>
```

**外送：**
```bash
mcd-cli order create --store <STORE_CODE> --be <BE_CODE> --address <ADDRESS_ID> --order-type 2 --be-type 2 \
  --items '[{"productCode":"<CODE>","quantity":1}]'
```

**团餐：**
```bash
mcd-cli order create --store <STORE_CODE> --be <BE_CODE> --order-type 2 --be-type 6 \
  --items '[{"productCode":"<CODE>","quantity":1}]' \
  --gm-service-code <GM_SERVICE_CODE>
```

**预约下单：**
```bash
mcd-cli order create --store <STORE_CODE> --order-type 1 --be-type 1 \
  --items '[{"productCode":"<CODE>","quantity":1}]' \
  --take-way <TAKE_WAY_CODE> \
  --reservation-date "2026-05-25 12:00"
```

### 6. 查询订单

```bash
mcd-cli order query <ORDER_ID>
```

### 7. 积分商城

**虚拟券兑换：**
```bash
mcd-cli mall products
mcd-cli mall detail <SPU_ID>
mcd-cli mall exchange --sku-id <SKU_ID> --count 1
```

**实物兑换：**
```bash
mcd-cli mall physical --sku-id <SKU_ID> --count 1 --address-id <ADDRESS_ID> --spu-category 2
```

**订单查询：**
```bash
mcd-cli mall orders
mcd-cli mall order-detail <ORDER_ID>
```

## 常用商品编码示例

| 商品 | 编码 |
|------|------|
| 板烧鸡腿堡三件套 | `9900005462` |
| 麦辣鸡腿汉堡三件套 | `9900005456` |
| 巨无霸三件套 | `9900005466` |
| 大薯条 | `4820` |
| 中杯可乐 | `903050` |

> 实际编码以 `mcd-cli menu` 查询结果为准，不同门店可能存在差异。

## 注意事项

- **到店取餐**（`orderType=1, beType=1`）不需要 `beCode` 和 `addressId`，但需要 `takeWayCode`。
- **外送**（`orderType=2, beType=2`）必须从 `delivery-query-addresses` 获取 `storeCode`、`beCode` 和 `addressId`，不可凭空生成。
- **得来速**（`beType=5`）与到店取餐类似，也需要 `takeWayCode`。
- **团餐**（`beType=6`）需要 `gmServiceCode`，可通过 `catering` 查询获取。
- **预约功能**：`menu`、`detail`、`price`、`order create` 均支持 `--reservation-date` 参数，格式 `yyyy-MM-dd HH:mm`。
- **优惠券**：`price` 和 `order create` 支持 `--coupon-id` 和 `--coupon-code`。
- 下单前务必先 `price` 确认价格。
- `payH5Url` 为扫码支付页，可在手机上打开完成支付，或在麦当劳 App「我的订单」中直接支付。
- 每个 Token 每分钟最多 600 次请求。

## 相关链接

- 麦当劳 MCP 平台：https://open.mcd.cn/mcp
- MCP 协议文档：https://modelcontextprotocol.io
