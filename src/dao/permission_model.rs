use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlQueryResult;
use std::clone::Clone;
use std::ops::Deref;
use std::rc::Rc;

// 引入全局变量
use sqlx::FromRow;
use crate::dao::db_pool;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub r#type: String,
    #[allow(non_snake_case)]
    pub parentId: Option<i64>,
    pub path: Option<String>,
    pub redirect: Option<String>,
    pub icon: Option<String>,
    pub component: Option<String>,
    pub layout: Option<String>,
    #[allow(non_snake_case)]
    pub keepAlive: Option<i8>,
    pub method: Option<String>,
    pub description: Option<String>,
    pub show: i8,
    pub enable: i8,
    pub order: i64,
}
impl Default for Permission {
    fn default() -> Self {
        Permission {
            id: 0,
            name: String::default(),
            code: String::default(),
            r#type: String::default(),
            parentId: Some(0),
            path: Some(String::default()),
            redirect: Some(String::default()),
            icon: Some(String::default()),
            component: Some(String::default()),
            layout: Some(String::default()),
            keepAlive: Some(0),
            method: Some(String::default()),
            description: Some(String::default()),
            show: 1,
            enable: 1,
            order: 0,
        }
    }
}
//
pub async fn find_1_level() -> Result<Vec<Permission>, sqlx::Error> {
    let pool = db_pool();

    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let rows: Vec<Permission> = sqlx::query_as::<_, Permission>(
        "SELECT * FROM `permission` WHERE parentId is NULL ORDER BY `order` ASC ",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// 查询1级权限通过 user_id
pub async fn find_1_level_where_by_user_id(user_id: i64) -> Result<Vec<Permission>, sqlx::Error> {
    let pool = db_pool();
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let rows: Vec<Permission> = sqlx::query_as::<_, Permission>("SELECT * FROM `permission` WHERE parentId is NULL and id in (select permissionId from role_permissions_permission where roleId IN(SELECT roleId FROM user_roles_role WHERE userId=?)) ORDER BY `order` ASC ")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}
// 查询下级权限通过 p_id
pub async fn find_all_where_by_p_id(p_id: i64) -> Result<Vec<Permission>, sqlx::Error> {
    let pool = db_pool();
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let rows: Vec<Permission> = sqlx::query_as::<_, Permission>(
        "SELECT * FROM `permission` WHERE parentId = ? ORDER BY `order` ASC ",
    )
    .bind(p_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_all() -> Result<Vec<Permission>, sqlx::Error> {
    let pool = db_pool();
    let rows: Vec<Permission> =
        sqlx::query_as::<_, Permission>("SELECT * FROM `permission` ORDER BY `order` ASC ")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

pub async fn find_all_where_by_user_id(user_id: i64) -> Result<Vec<Permission>, sqlx::Error> {
    let pool = db_pool();
    let rows: Vec<Permission> = sqlx::query_as::<_, Permission>(
        "SELECT DISTINCT p.* FROM `permission` p \
         INNER JOIN role_permissions_permission rpp ON rpp.permissionId = p.id \
         INNER JOIN user_roles_role urr ON urr.roleId = rpp.roleId \
         WHERE urr.userId = ? \
         ORDER BY p.`order` ASC ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
