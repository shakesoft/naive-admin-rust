use dotenv::dotenv;
use std::env;
use sqlx::mysql::MySqlPoolOptions;
use std::sync::{OnceLock};
use tracing::log::info;

pub mod permission_model;
pub mod profile_model;
pub mod role_model;
pub mod role_permissions_permission;
pub mod user_model;
pub mod user_roles_role_model;

// 定义懒加载全局变量，“懒加载里套懒加载”显得多余了
// lazy_static! {
//     pub static ref DB_POOL: OnceLock<sqlx::MySqlPool> = OnceLock::new();
// }

static DB_POOL: OnceLock<sqlx::MySqlPool> = OnceLock::new();

// 连接数据库
async fn init_pool() -> Result<sqlx::MySqlPool, sqlx::Error> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new().connect(&database_url).await?;
    Ok(pool)
}

// 释放连接池
pub async fn mysql_disconnect() -> Result<(), sqlx::Error> {
    // if let Some(pool) = DB_POOL.lock().unwrap().take() {
    //     pool.close().await;
    // }
    if let Some(pool) = DB_POOL.get(){
        //DB_POOL 依然存在（是全局的 OnceLock），但它指向的连接池已经被关闭。
        //关闭时能等待所有连接释放，比单纯依赖进程退出要干净。
        info!("正在安全关闭数据库连接...");
        let pool = pool.clone();
        pool.close().await;
        info!("DB_POOL指向的连接池已经被关闭");
    }
    Ok(())
}

pub async fn mysql_connect() {
    // 初始化连接池
    let pool = init_pool().await.unwrap();
    // 存储连接池到全局变量
    // *DB_POOL.lock().unwrap() = Some(pool);
    DB_POOL.get_or_init(move || pool);
}

pub fn db_pool() -> &'static sqlx::MySqlPool {
    DB_POOL.get().expect("DB_POOL 尚未初始化，请先调用 init_db_pool()")
}

pub async fn check_db_pool_status() -> bool {
    if let Some(pool) = DB_POOL.get() {
        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .is_ok()
    } else {
        false
    }
}
