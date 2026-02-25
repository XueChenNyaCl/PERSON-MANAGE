🚀 方案一：使用 tokio-postgres 异步驱动
这是最直接、最灵活的方式，适合需要精细控制SQL和执行异步操作的场景 。

1. 添加依赖
首先，在你的 Cargo.toml 文件中添加必要的依赖：

toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7"
这里 tokio 是异步运行时，tokio-postgres 则是基于它的PostgreSQL客户端 。

2. 编写代码 (main.rs)
以下是一个完整的示例，演示了如何连接数据库、创建表、插入数据和查询数据。

rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. 配置数据库连接字符串
    // 请将 your_username, your_password, your_dbname 替换为你实际的数据库信息
    let conn_str = "host=localhost user=your_username password=your_password dbname=your_dbname";
    
    // 2. 尝试连接数据库
    let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;

    // 3. 启动一个后台任务来处理数据库连接的生命周期
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("数据库连接错误: {}", e);
        }
    });

    // 4. 执行SQL：创建一个示例表 (如果不存在)
    client.execute(
        "CREATE TABLE IF NOT EXISTS person (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            age INT NOT NULL
        )",
        &[],
    ).await?;
    println!("✅ 数据表 'person' 已就绪。");

    // 5. 插入数据 (使用参数化查询，防止SQL注入)
    client.execute(
        "INSERT INTO person (name, age) VALUES ($1, $2)",
        &[&"Alice", &30],
    ).await?;
    client.execute(
        "INSERT INTO person (name, age) VALUES ($1, $2)",
        &[&"Bob", &25],
    ).await?;
    println!("✅ 示例数据插入成功。");

    // 6. 查询数据
    let rows = client.query("SELECT id, name, age FROM person", &[]).await?;

    // 7. 处理查询结果
    println!("📊 查询结果:");
    for row in rows {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let age: i32 = row.get("age");
        println!("   - id: {}, name: {}, age: {}", id, name, age);
    }

    Ok(())
}
代码解读：

连接：connect 方法返回一个 client 用于执行操作，以及一个 connection 对象，后者需要在后台运行以维持与服务器的通信 。

参数化查询：使用 $1, $2 这样的占位符来传递参数，这是避免SQL注入攻击的标准做法 。

获取数据：通过 row.get() 方法并指定列名或索引来获取值，需要显式声明你期望的Rust类型 。

🏗️ 方案二：使用 Diesel ORM 框架
如果你的项目规模较大，或者希望以更结构化的方式与数据库交互，Diesel是一个非常好的选择。它是一个类型安全、编译时检查的ORM 。

1. 添加依赖并安装CLI工具
在 Cargo.toml 中添加：

toml
[dependencies]
diesel = { version = "1.4", features = ["postgres"] }
dotenv = "0.15"
然后，安装 Diesel CLI 工具来帮助管理数据库迁移：

bash
cargo install diesel_cli --no-default-features --features postgres
2. 配置数据库连接
在项目根目录创建 .env 文件，写入你的数据库连接信息：

env
DATABASE_URL=postgres://your_username:your_password@localhost/your_dbname
3. 初始化Diesel并创建迁移
运行以下命令来设置Diesel，它会创建 migrations 目录并生成初始schema文件 。

bash
diesel setup
创建一个新的迁移来建立数据表：

bash
diesel migration generate create_posts
在生成的 migrations/创建时间_create_posts/up.sql 中编写建表语句：

sql
CREATE TABLE posts (
  id SERIAL PRIMARY KEY,
  title VARCHAR NOT NULL,
  body TEXT NOT NULL,
  published BOOLEAN NOT NULL DEFAULT 'f'
)
在对应的 down.sql 中编写回滚语句：

sql
DROP TABLE posts
最后，执行迁移以实际创建表：

bash
diesel migration run
4. 编写Rust代码进行读写
Diesel会根据你的表结构自动生成一部分代码（在 src/schema.rs 中）。你需要定义与表结构对应的Rust结构体 。

rust
// src/models.rs
use crate::schema::posts;

// 用于查询的数据结构
#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

// 用于插入新数据的数据结构
#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}
rust
// src/main.rs
#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    use self::schema::posts::dsl::*;
    let connection = establish_connection();

    // 插入一篇新文章
    let new_post = models::NewPost {
        title: "我的第一篇博客",
        body: "使用Diesel操作PostgreSQL真的很简单！",
    };
    
    diesel::insert_into(posts::table)
        .values(&new_post)
        .get_result::<models::Post>(&connection)
        .expect("Error saving new post");
    println!("✅ 文章已保存。");

    // 查询并显示所有已发布的文章
    let results = posts
        .filter(published.eq(true))
        .limit(5)
        .load::<models::Post>(&connection)
        .expect("Error loading posts");

    println!("📊 显示 {} 篇已发布的文章:", results.len());
    for post in results {
        println!("   - {}: {}", post.title, post.body);
    }
}