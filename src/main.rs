mod config;
mod fmt;
mod mcp;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use mcp::McpClient;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "mcd-cli")]
#[command(about = "麦当劳 MCP CLI - 基于麦当劳官方 MCP Server 的点餐工具")]
struct Cli {
    #[arg(long, env = "MCD_MCP_TOKEN", help = "MCP Token")]
    token: Option<String>,

    #[arg(long, env = "MCD_MCP_URL", default_value = "https://mcp.mcd.cn", help = "MCP Server URL")]
    url: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 初始化并测试连接
    Init,
    /// 查看活动日历
    Calendar,
    /// 查看当前时间
    Time,
    /// 查看我的账户（含积分）
    Account,
    /// 查询附近门店
    Nearby {
        #[arg(long, default_value = "2", help = "beType: 1=到店, 2=麦乐送")]
        be_type: i32,
        #[arg(long, default_value = "2", help = "searchType: 1=收藏, 2=按位置")]
        search_type: i32,
        #[arg(long, help = "城市名")]
        city: Option<String>,
        #[arg(long, help = "关键词")]
        keyword: Option<String>,
    },
    /// 地址管理
    Address {
        #[command(subcommand)]
        action: AddressCommands,
    },
    /// 浏览菜单
    Menu {
        #[arg(long, help = "门店 storeCode")]
        store: String,
        #[arg(long, help = "门店 beCode（外送必填）")]
        be: Option<String>,
        #[arg(long, default_value = "2", help = "订单类型: 1=到店取餐, 2=外送")]
        order_type: i32,
    },
    /// 餐品详情
    Detail {
        #[arg(help = "商品 code")]
        code: String,
        #[arg(long, help = "门店 storeCode")]
        store: String,
        #[arg(long, help = "门店 beCode（外送必填）")]
        be: Option<String>,
        #[arg(long, default_value = "2", help = "订单类型: 1=到店取餐, 2=外送")]
        order_type: i32,
    },
    /// 优惠券
    Coupon {
        #[command(subcommand)]
        action: CouponCommands,
    },
    /// 积分商城
    Mall {
        #[command(subcommand)]
        action: MallCommands,
    },
    /// 计算价格
    Price {
        #[arg(long, help = "门店 storeCode")]
        store: String,
        #[arg(long, help = "门店 beCode（外送必填）")]
        be: Option<String>,
        #[arg(long, default_value = "2", help = "订单类型: 1=到店取餐, 2=外送")]
        order_type: i32,
        #[arg(long, help = "商品列表 JSON，如 [{\"productCode\":\"xxx\",\"quantity\":1}]")]
        items: String,
    },
    /// 创建订单
    Order {
        #[command(subcommand)]
        action: OrderCommands,
    },
    /// 登录并保存 Token 到配置文件
    Login {
        #[arg(long, help = "MCP Token")]
        token: String,
        #[arg(long, help = "MCP Server URL")]
        url: Option<String>,
    },
    /// 查看配置文件路径和当前配置
    Config,
    /// 交互式点单模式
    Interactive,
}

#[derive(Subcommand, Debug)]
enum AddressCommands {
    /// 查询配送地址
    List,
    /// 新增配送地址
    Add,
}

#[derive(Subcommand, Debug)]
enum CouponCommands {
    /// 门店可用优惠券
    Store {
        #[arg(long, help = "门店 storeCode")]
        store: String,
        #[arg(long, help = "门店 beCode")]
        be: String,
        #[arg(long, default_value = "2", help = "订单类型: 1=到店取餐, 2=外送")]
        order_type: i32,
    },
    /// 我的优惠券
    My,
    /// 可领优惠券列表
    Available,
    /// 一键领券
    Receive,
}

#[derive(Subcommand, Debug)]
enum MallCommands {
    /// 积分兑换商品列表
    Products,
    /// 积分兑换商品详情
    Detail {
        #[arg(help = "商品 spuId")]
        spu_id: i64,
    },
    /// 积分兑换下单
    Exchange {
        #[arg(long, help = "商品 skuId")]
        sku_id: i64,
        #[arg(long, default_value = "1", help = "兑换数量")]
        count: i32,
    },
}

#[derive(Subcommand, Debug)]
enum OrderCommands {
    /// 创建订单
    Create {
        #[arg(long, help = "门店 storeCode")]
        store: String,
        #[arg(long, help = "门店 beCode（外送必填）")]
        be: Option<String>,
        #[arg(long, help = "地址 ID（外送必填）")]
        address: Option<String>,
        #[arg(long, help = "商品列表 JSON")]
        items: String,
        #[arg(long, default_value = "2", help = "订单类型: 1=到店取餐, 2=外送")]
        order_type: i32,
        #[arg(long, help = "取餐方式编码（orderType=1时必传，从calculate-price获取）")]
        take_way: Option<String>,
    },
    /// 查询订单
    Query {
        #[arg(help = "订单号 orderId")]
        id: String,
    },
}

fn read_line(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn read_line_opt(prompt: &str) -> Option<String> {
    let s = read_line(prompt).ok()?;
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn print_divider() {
    println!("{}", "-".repeat(50));
}

fn print_result(result: &mcp::ToolResult) {
    if let Some(ref structured) = result.structured_content {
        fmt::pretty_print(structured);
        return;
    }
    let text = result
        .content
        .iter()
        .filter_map(|c| c.text.clone())
        .collect::<Vec<_>>()
        .join("");
    fmt::pretty_print(&Value::String(text));
}

async fn run_init(client: &McpClient) -> Result<()> {
    println!("正在连接麦当劳 MCP Server...");
    let result = client.initialize().await?;
    println!("✅ 连接成功!");
    println!("   协议版本: {}", result.protocol_version);
    println!("   服务端: {} v{}", result.server_info.name, result.server_info.version);
    Ok(())
}

async fn run_calendar(client: &McpClient) -> Result<()> {
    let result = client.call_tool("campaign-calendar", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_time(client: &McpClient) -> Result<()> {
    let result = client.call_tool("now-time-info", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_account(client: &McpClient) -> Result<()> {
    let result = client.call_tool("query-my-account", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_nearby(
    client: &McpClient,
    be_type: i32,
    search_type: i32,
    city: Option<&str>,
    keyword: Option<&str>,
) -> Result<()> {
    let mut args = serde_json::json!({
        "beType": be_type,
        "searchType": search_type,
    });
    if let Some(c) = city {
        args["city"] = Value::String(c.to_string());
    }
    if let Some(k) = keyword {
        args["keyword"] = Value::String(k.to_string());
    }
    let result = client.call_tool("query-nearby-stores", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_address_list(client: &McpClient) -> Result<()> {
    let result = client
        .call_tool("delivery-query-addresses", serde_json::json!({"beType": 2}))
        .await?;
    print_result(&result);
    Ok(())
}

async fn run_address_add(client: &McpClient) -> Result<()> {
    println!("请输入配送地址信息（麦乐送 beType=2）:");
    let city = read_line("城市名称: ")?;
    let contact_name = read_line("联系人姓名: ")?;
    let phone = read_line("联系电话: ")?;
    let address = read_line("配送地址（小区/楼栋）: ")?;
    let address_detail = read_line("门牌号: ")?;
    let gender = read_line_opt("性别（先生/女士，回车跳过）: ");

    let mut args = serde_json::json!({
        "city": city,
        "contactName": contact_name,
        "phone": phone,
        "address": address,
        "addressDetail": address_detail,
        "beType": 2
    });
    if let Some(g) = gender {
        args["gender"] = Value::String(g);
    }

    let result = client.call_tool("delivery-create-address", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_menu(client: &McpClient, store: &str, be: Option<&str>, order_type: i32) -> Result<()> {
    let mut args = serde_json::json!({
        "storeCode": store,
        "orderType": order_type,
    });
    if let Some(b) = be {
        args["beCode"] = Value::String(b.to_string());
    }
    let result = client.call_tool("query-meals", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_detail(
    client: &McpClient,
    code: &str,
    store: &str,
    be: Option<&str>,
    order_type: i32,
) -> Result<()> {
    let mut args = serde_json::json!({
        "code": code,
        "storeCode": store,
        "orderType": order_type,
    });
    if let Some(b) = be {
        args["beCode"] = Value::String(b.to_string());
    }
    let result = client.call_tool("query-meal-detail", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_coupon_store(
    client: &McpClient,
    store: &str,
    be: &str,
    order_type: i32,
) -> Result<()> {
    let args = serde_json::json!({
        "storeCode": store,
        "beCode": be,
        "orderType": order_type,
    });
    let result = client.call_tool("query-store-coupons", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_coupon_my(client: &McpClient) -> Result<()> {
    let result = client.call_tool("query-my-coupons", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_coupon_available(client: &McpClient) -> Result<()> {
    let result = client.call_tool("available-coupons", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_coupon_receive(client: &McpClient) -> Result<()> {
    let result = client.call_tool("auto-bind-coupons", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_mall_products(client: &McpClient) -> Result<()> {
    let result = client.call_tool("mall-points-products", serde_json::json!({})).await?;
    print_result(&result);
    Ok(())
}

async fn run_mall_detail(client: &McpClient, spu_id: i64) -> Result<()> {
    let args = serde_json::json!({"spuId": spu_id});
    let result = client.call_tool("mall-product-detail", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_mall_exchange(client: &McpClient, sku_id: i64, count: i32) -> Result<()> {
    let args = serde_json::json!({"skuId": sku_id, "count": count});
    let result = client.call_tool("mall-create-order", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_calculate_price(
    client: &McpClient,
    store: &str,
    be: Option<&str>,
    order_type: i32,
    items: &str,
) -> Result<()> {
    let items_val: Value = serde_json::from_str(items)?;
    let mut args = serde_json::json!({
        "storeCode": store,
        "orderType": order_type,
        "items": items_val,
    });
    if let Some(b) = be {
        args["beCode"] = Value::String(b.to_string());
    }
    let result = client.call_tool("calculate-price", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_order_create(
    client: &McpClient,
    store: &str,
    be: Option<&str>,
    address: Option<&str>,
    items: &str,
    order_type: i32,
    take_way: Option<&str>,
) -> Result<()> {
    let items_val: Value = serde_json::from_str(items)?;
    let mut args = serde_json::json!({
        "storeCode": store,
        "items": items_val,
        "orderType": order_type,
    });
    if order_type == 2 {
        if let Some(b) = be {
            args["beCode"] = Value::String(b.to_string());
        }
        if let Some(a) = address {
            args["addressId"] = Value::String(a.to_string());
        }
    }
    if let Some(tw) = take_way {
        args["takeWayCode"] = Value::String(tw.to_string());
    }
    let result = client.call_tool("create-order", args).await?;
    print_result(&result);
    Ok(())
}

async fn run_order_query(client: &McpClient, id: &str) -> Result<()> {
    let args = serde_json::json!({"orderId": id});
    let result = client.call_tool("query-order", args).await?;
    print_result(&result);
    Ok(())
}

async fn interactive_mode(client: &McpClient) -> Result<()> {
    loop {
        print_divider();
        println!("🍟 麦当劳 CLI 交互模式");
        print_divider();
        println!("1.  测试连接");
        println!("2.  查看活动日历");
        println!("3.  查看当前时间");
        println!("4.  查看我的账户/积分");
        println!("5.  查询附近门店");
        println!("6.  查看配送地址");
        println!("7.  新增配送地址");
        println!("8.  浏览菜单");
        println!("9.  查看餐品详情");
        println!("10. 门店优惠券");
        println!("11. 我的优惠券 / 一键领券 / 可领券");
        println!("12. 积分商城");
        println!("13. 快速点单（计算价格+下单）");
        println!("14. 查询订单详情");
        println!("0.  退出");
        print_divider();

        let choice = read_line("请输入选项: ")?;
        match choice.as_str() {
            "1" => {
                if let Err(e) = run_init(client).await {
                    println!("❌ 连接失败: {}", e);
                }
            }
            "2" => {
                if let Err(e) = run_calendar(client).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "3" => {
                if let Err(e) = run_time(client).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "4" => {
                if let Err(e) = run_account(client).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "5" => {
                let city = read_line("城市名: ")?;
                let keyword = read_line("关键词（商圈/学校/路名）: ")?;
                if let Err(e) = run_nearby(client, 1, 2, Some(&city), Some(&keyword)).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "6" => {
                if let Err(e) = run_address_list(client).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "7" => {
                if let Err(e) = run_address_add(client).await {
                    println!("❌ 添加失败: {}", e);
                }
            }
            "8" => {
                let store = read_line("门店 storeCode: ")?;
                let be = read_line_opt("门店 beCode（到店取餐回车跳过）: ");
                let ot = read_line("订单类型 (1=到店, 2=外送): ")?;
                let order_type: i32 = ot.parse().unwrap_or(2);
                if let Err(e) = run_menu(client, &store, be.as_deref(), order_type).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "9" => {
                let code = read_line("商品 code: ")?;
                let store = read_line("门店 storeCode: ")?;
                let be = read_line_opt("门店 beCode（到店取餐回车跳过）: ");
                let ot = read_line("订单类型 (1=到店, 2=外送): ")?;
                let order_type: i32 = ot.parse().unwrap_or(2);
                if let Err(e) = run_detail(client, &code, &store, be.as_deref(), order_type).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "10" => {
                let store = read_line("门店 storeCode: ")?;
                let be = read_line("门店 beCode: ")?;
                let ot = read_line("订单类型 (1=到店, 2=外送): ")?;
                let order_type: i32 = ot.parse().unwrap_or(2);
                if let Err(e) = run_coupon_store(client, &store, &be, order_type).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "11" => {
                println!("a. 我的优惠券");
                println!("b. 可领优惠券列表");
                println!("c. 一键领券");
                let sub = read_line("选择: ")?;
                match sub.as_str() {
                    "a" => {
                        if let Err(e) = run_coupon_my(client).await {
                            println!("❌ 查询失败: {}", e);
                        }
                    }
                    "b" => {
                        if let Err(e) = run_coupon_available(client).await {
                            println!("❌ 查询失败: {}", e);
                        }
                    }
                    "c" => {
                        if let Err(e) = run_coupon_receive(client).await {
                            println!("❌ 领券失败: {}", e);
                        }
                    }
                    _ => println!("无效选择"),
                }
            }
            "12" => {
                println!("a. 积分兑换商品列表");
                println!("b. 积分兑换商品详情");
                println!("c. 积分兑换下单");
                let sub = read_line("选择: ")?;
                match sub.as_str() {
                    "a" => {
                        if let Err(e) = run_mall_products(client).await {
                            println!("❌ 查询失败: {}", e);
                        }
                    }
                    "b" => {
                        let spu = read_line("商品 spuId: ")?;
                        let spu_id: i64 = spu.parse().unwrap_or(0);
                        if let Err(e) = run_mall_detail(client, spu_id).await {
                            println!("❌ 查询失败: {}", e);
                        }
                    }
                    "c" => {
                        let sku = read_line("商品 skuId: ")?;
                        let sku_id: i64 = sku.parse().unwrap_or(0);
                        let cnt = read_line("兑换数量（默认1）: ")?;
                        let count: i32 = cnt.parse().unwrap_or(1);
                        if let Err(e) = run_mall_exchange(client, sku_id, count).await {
                            println!("❌ 兑换失败: {}", e);
                        }
                    }
                    _ => println!("无效选择"),
                }
            }
            "13" => {
                println!("--- 快速点单流程 ---");
                println!("提示: 门店信息请先从【查看配送地址】或【查询附近门店】中获取");
                let store = read_line("门店 storeCode: ")?;
                let be = read_line_opt("门店 beCode（到店取餐回车跳过）: ");
                let address = read_line_opt("地址 ID（到店取餐回车跳过）: ");
                let ot = read_line("订单类型 (1=到店, 2=外送): ")?;
                let order_type: i32 = ot.parse().unwrap_or(2);
                let items = read_line(
                    "商品列表 JSON: [{\"productCode\":\"xxx\",\"quantity\":1}]: ",
                )?;
                println!("\n正在计算价格...");
                if let Err(e) =
                    run_calculate_price(client, &store, be.as_deref(), order_type, &items).await
                {
                    println!("❌ 计价失败: {}", e);
                    continue;
                }
                let confirm = read_line("确认下单? (y/n): ")?;
                if confirm.eq_ignore_ascii_case("y") {
                    println!("\n正在创建订单...");
                    let take_way = if order_type == 1 {
                        read_line_opt("取餐方式编码 takeWayCode（从calculate-price结果获取，回车跳过）: ")
                    } else {
                        None
                    };
                    if let Err(e) = run_order_create(
                        client,
                        &store,
                        be.as_deref(),
                        address.as_deref(),
                        &items,
                        order_type,
                        take_way.as_deref(),
                    )
                    .await
                    {
                        println!("❌ 下单失败: {}", e);
                    }
                } else {
                    println!("已取消下单");
                }
            }
            "14" => {
                let id = read_line("订单号 orderId: ")?;
                if let Err(e) = run_order_query(client, &id).await {
                    println!("❌ 查询失败: {}", e);
                }
            }
            "0" | "q" | "quit" | "exit" => {
                println!("再见，祝您用餐愉快! 🍔");
                break;
            }
            _ => println!("无效选项，请重新输入"),
        }
    }
    Ok(())
}

fn resolve_token(cli_token: Option<String>, cfg: &Config) -> Result<String> {
    cli_token
        .or_else(|| std::env::var("MCD_MCP_TOKEN").ok())
        .or_else(|| cfg.token.clone())
        .context("错误: 需要提供 MCP Token\n方式1: mcd-cli login --token xxx\n方式2: 环境变量 MCD_MCP_TOKEN=xxx\n方式3: 命令行参数 --token xxx")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load().context("加载配置文件失败")?;

    match cli.command {
        Some(Commands::Login { token, url }) => {
            let mut cfg = Config::load()?;
            cfg.set_token(token);
            if let Some(u) = url {
                cfg.set_url(u);
            }
            cfg.save()?;
            println!("✅ Token 已保存到: {}", Config::config_path()?.display());
            return Ok(());
        }
        Some(Commands::Config) => {
            let cfg = Config::load()?;
            println!("配置文件路径: {}", Config::config_path()?.display());
            println!("token: {}", cfg.token.as_deref().unwrap_or("(未设置)"));
            println!("url:   {}", cfg.url.as_deref().unwrap_or("(默认 https://mcp.mcd.cn)"));
            return Ok(());
        }
        _ => {}
    }

    let token = resolve_token(cli.token, &cfg)?;
    let url = cfg.url.unwrap_or(cli.url);
    let client = McpClient::with_url(&url, token)?;

    match cli.command {
        Some(Commands::Init) => run_init(&client).await?,
        Some(Commands::Calendar) => run_calendar(&client).await?,
        Some(Commands::Time) => run_time(&client).await?,
        Some(Commands::Account) => run_account(&client).await?,
        Some(Commands::Nearby { be_type, search_type, city, keyword }) => {
            run_nearby(&client, be_type, search_type, city.as_deref(), keyword.as_deref()).await?
        }
        Some(Commands::Address { action }) => match action {
            AddressCommands::List => run_address_list(&client).await?,
            AddressCommands::Add => run_address_add(&client).await?,
        },
        Some(Commands::Menu { store, be, order_type }) => {
            run_menu(&client, &store, be.as_deref(), order_type).await?
        }
        Some(Commands::Detail { code, store, be, order_type }) => {
            run_detail(&client, &code, &store, be.as_deref(), order_type).await?
        }
        Some(Commands::Coupon { action }) => match action {
            CouponCommands::Store { store, be, order_type } => {
                run_coupon_store(&client, &store, &be, order_type).await?
            }
            CouponCommands::My => run_coupon_my(&client).await?,
            CouponCommands::Available => run_coupon_available(&client).await?,
            CouponCommands::Receive => run_coupon_receive(&client).await?,
        },
        Some(Commands::Mall { action }) => match action {
            MallCommands::Products => run_mall_products(&client).await?,
            MallCommands::Detail { spu_id } => run_mall_detail(&client, spu_id).await?,
            MallCommands::Exchange { sku_id, count } => {
                run_mall_exchange(&client, sku_id, count).await?
            }
        },
        Some(Commands::Price { store, be, order_type, items }) => {
            run_calculate_price(&client, &store, be.as_deref(), order_type, &items).await?
        }
        Some(Commands::Order { action }) => match action {
            OrderCommands::Create { store, be, address, items, order_type, take_way } => {
                run_order_create(
                    &client,
                    &store,
                    be.as_deref(),
                    address.as_deref(),
                    &items,
                    order_type,
                    take_way.as_deref(),
                )
                .await?
            }
            OrderCommands::Query { id } => run_order_query(&client, &id).await?,
        },
        Some(Commands::Interactive) | None => {
            println!("🍟 麦当劳 MCP CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("正在进入交互模式...\n");
            interactive_mode(&client).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
