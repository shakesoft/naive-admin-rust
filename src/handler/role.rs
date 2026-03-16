use std::ops::Deref;
use std::collections::HashMap;
use crate::tools;
use crate::{
    dto::resp::ApiResponse,
    dto::{comm_api, role_api, user_api},
    dao::{
        permission_model, profile_model, role_model, role_permissions_permission, user_model,
        user_roles_role_model,
    },
};
use axum::{
    extract::{Extension, Json, Path, Query, Request},
    middleware::{self, Next},
};
use chrono::Utc;
use sqlx::MySqlPool;
use std::rc::Rc;
use std::time::Instant;
use aspect_macros::aspect;
use tracing::info;
use validator::Validate;
use crate::aop::aspects::timer::Timer;
use crate::dao::db_pool;

fn build_permission_tree(
    parent_id: Option<i64>,
    grouped_permissions: &mut HashMap<Option<i64>, Vec<role_api::PermissionItem>>,
) -> Vec<Box<role_api::PermissionItem>> {
    grouped_permissions
        .remove(&parent_id)
        .unwrap_or_default()
        .into_iter()
        .map(|mut item| {
            let children = build_permission_tree(Some(item.id), grouped_permissions);
            item.children = Some(children);
            Box::new(item)
        })
        .collect()
}

// 所有角色
#[aspect(Timer)]
pub async fn all(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
) -> Json<ApiResponse<Vec<role_model::Role>>> {
    return match role_model::fetch_all_role().await {
        Ok(a) => Json(ApiResponse::succ(Some(a))),
        Err(err) => Json(ApiResponse::err(&format!("获取所有权限失败:{:?}", err))),
    };
}

pub async fn test(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
) -> Json<ApiResponse<String>> {
    // let start = Instant::now();
    // let pool = db_pool();
    // let duration = start.elapsed();
    // info!("代码运行耗时: {:?}", duration);
    Json(ApiResponse::new(200, None, &format!("{}", "成功")))
}

// 新增角色
pub async fn add_role(Json(req): Json<role_api::RoleAddReq>) -> Json<ApiResponse<String>> {
    if let Err(error) = req.validate() {
        return Json(ApiResponse::new(400, None, &format!("{}", error)));
    }
    let pool = db_pool();
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return Json(ApiResponse::err(&format!("开启事务失败:{:?}", err))),
    };
    let add_data = role_model::Role {
        id: 0,
        code: req.code,
        name: req.name,
        enable: req.enable as i64,
    };
    let new_role_id = match role_model::add_by_struct(&mut tx, add_data.clone()).await {
        Ok(id) => id,
        Err(err) => {
            if let Err(rollback_err) = tx.rollback().await {
                return Json(ApiResponse::err(&format!(
                    "事务回滚失败: {:?}",
                    rollback_err
                )));
            }
            return Json(ApiResponse::err(&format!("新角色失败:{:?}", err)));
        }
    };
    if let Some(pmids) = req.permissionIds {
        // 新增角色-资源权限关联
        for pmid in pmids {
            let add_data = role_permissions_permission::RolePermissionsPermission {
                permissionId: pmid,
                roleId: new_role_id as i64,
            };
            match role_permissions_permission::add_role_permissions_by_struct(
                &mut tx,
                add_data.clone(),
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    if let Err(rollback_err) = tx.rollback().await {
                        return Json(ApiResponse::err(&format!(
                            "事务回滚失败: {:?}",
                            rollback_err
                        )));
                    }
                    return Json(ApiResponse::err(&format!(
                        "新增角色-资源权限失败:{:?}",
                        err
                    )));
                }
            };
        }
    }
    if let Err(commit_err) = tx.commit().await {
        return Json(ApiResponse::err(&format!("事务提交失败: {:?}", commit_err)));
    }
    return Json(ApiResponse::succ(Some("ok".to_string())));
}

// 当前用户权限树
pub async fn permissions_tree(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
) -> Json<ApiResponse<Option<Vec<role_api::PermissionItem>>>> {
    let uid = curr_user.id;
    let is_admin_result = user_roles_role_model::find_is_admin_role_by_user_id(uid).await;
    let is_admin = match is_admin_result {
        Ok(a) => a,
        Err(err) => {
            let error_msg = format!("获取用户admin权限信息失败:{:?}", err);
            return Json(ApiResponse::err(&error_msg));
        }
    };
    let permissions_result = if is_admin {
        permission_model::find_all().await
    } else {
        permission_model::find_all_where_by_user_id(uid).await
    };
    let permissions = match permissions_result {
        Ok(rows) => rows,
        Err(err) => {
            let error_msg = if is_admin {
                format!("获取所有权限信息失败:{:?}", err)
            } else {
                format!("获取用户权限信息失败:{:?}", err)
            };
            return Json(ApiResponse::err(&error_msg));
        }
    };

    let mut grouped_permissions: HashMap<Option<i64>, Vec<role_api::PermissionItem>> =
        HashMap::new();
    for permission in permissions {
        grouped_permissions
            .entry(permission.parentId)
            .or_default()
            .push(role_api::PermissionItem {
                id: permission.id,
                name: permission.name,
                code: permission.code,
                r#type: permission.r#type,
                parentId: permission.parentId,
                path: permission.path,
                redirect: permission.redirect,
                icon: permission.icon,
                component: permission.component,
                layout: permission.layout,
                keepAlive: permission.keepAlive,
                method: permission.method,
                description: permission.description,
                show: permission.show,
                enable: permission.enable,
                order: permission.order,
                children: Some(Vec::new()),
            });
    }

    let rp_arr = build_permission_tree(None, &mut grouped_permissions)
        .into_iter()
        .map(|item| *item)
        .collect();
    return Json(ApiResponse::succ(Some(Some(rp_arr))));
}

// 角色列表
#[aspect(Timer)]
pub async fn page_list(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
    req: Query<role_api::RolePageReq>,
) -> Json<ApiResponse<role_api::RolePageResp>> {
    if let Err(error) = req.validate() {
        return Json(ApiResponse::new(400, None, &format!("{}", error)));
    }
    let result = role_model::fetch_all_by_req(req).await;
    let all_role = match result {
        Ok(u) => u,
        Err(err) => return Json(ApiResponse::err(&format!("获取列表信息失败:{:?}", err))),
    };
    let mut list_item = Vec::new();
    for ro in all_role {
        let mut tmp = role_api::RolePageItem::default();
        tmp.id = ro.id;
        tmp.name = ro.name;
        tmp.code = ro.code;
        tmp.enable = ro.enable != 0;
        // 获取 permission ids
        let pmids_result =
            role_permissions_permission::fetch_permission_ids_where_role_id(tmp.id).await;
        let perm_ids = match pmids_result {
            Ok(rows) => {
                if !rows.is_empty() {
                    rows
                } else {
                    Vec::new()
                }
            }
            Err(err) => {
                return Json(ApiResponse::err(&format!(
                    "获取角色菜单权限列表失败:{:?}",
                    err
                )))
            }
        };
        tmp.permissionIds = Some(perm_ids);
        list_item.push(tmp)
    }
    let mut rp = role_api::RolePageResp {
        pageData: Some(list_item),
    };
    return Json(ApiResponse::succ(Some(rp)));
}

// 角色更新：状态禁用/开启+编辑
pub async fn patch_role(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<role_api::RolePatchReq>,
) -> Json<ApiResponse<String>> {
    if let Err(error) = req.validate() {
        return Json(ApiResponse::new(400, None, &format!("{}", error)));
    }

    // 更新状态禁用/开启
    if req.name.is_none() {
        match role_model::update_enable_by_id(req.enable, id).await {
            Ok(_) => {}
            Err(err) => return Json(ApiResponse::err(&format!("更新角色状态失败:{:?}", err))),
        }
        return Json(ApiResponse::succ(Some("ok".to_string())));
    }
    let pool = db_pool();
    // 编辑
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return Json(ApiResponse::err(&format!("开启事务失败:{:?}", err))),
    };
    // 修改 role 表
    let role_data = role_model::Role {
        id,
        code: req.code.unwrap_or(String::new()),
        name: req.name.unwrap_or(String::new()),
        enable: req.enable as i64,
    };
    match role_model::update_role_by_struct(&mut tx, role_data.clone()).await {
        Ok(_) => {}
        Err(err) => {
            return Json(ApiResponse::err(&format!("修改角色信息失败:{:?}", err)));
        }
    };
    // 角色-资源关系表（先删后增)
    match role_permissions_permission::delete_permissions_by_role_id(&mut tx, id).await {
        Ok(_) => {}
        Err(err) => {
            if let Err(rollback_err) = tx.rollback().await {
                return Json(ApiResponse::err(&format!(
                    "事务回滚失败: {:?}",
                    rollback_err
                )));
            }
            return Json(ApiResponse::err(&format!(
                "删除角色-资源权限失败:{:?}",
                err
            )));
        }
    };
    if let Some(pmids) = req.permissionIds {
        // 新增角色-资源权限关联
        for pmid in pmids {
            let add_data = role_permissions_permission::RolePermissionsPermission {
                permissionId: pmid as i64,
                roleId: id,
            };
            match role_permissions_permission::add_role_permissions_by_struct(
                &mut tx,
                add_data.clone(),
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    if let Err(rollback_err) = tx.rollback().await {
                        return Json(ApiResponse::err(&format!(
                            "事务提交失败: {:?}",
                            rollback_err
                        )));
                    }
                    return Json(ApiResponse::err(&format!(
                        "新增角色-资源权限失败:{:?}",
                        err
                    )));
                }
            };
        }
    }

    if let Err(commit_err) = tx.commit().await {
        return Json(ApiResponse::err(&format!("事务提交失败: {:?}", commit_err)));
    }
    return Json(ApiResponse::succ(Some("ok".to_string())));
}

// 角色绑定用户
pub async fn add_user(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<role_api::RoleAddUserReq>,
) -> Json<ApiResponse<String>> {
    if let Err(error) = req.validate() {
        return Json(ApiResponse::new(400, None, &format!("{}", error)));
    }
    let pool = db_pool();
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return Json(ApiResponse::err(&format!("开启事务失败:{:?}", err))),
    };
    for uid in req.userIds {
        let add_data = user_roles_role_model::UserRolesRole {
            userId: uid,
            roleId: id,
        };
        match user_roles_role_model::add_user_role_by_struct(&mut tx, add_data.clone()).await {
            Ok(_) => {}
            Err(err) => {
                if let Err(rollback_err) = tx.rollback().await {
                    return Json(ApiResponse::err(&format!(
                        "事务提交失败: {:?}",
                        rollback_err
                    )));
                }
                return Json(ApiResponse::err(&format!("新增用户角色失败:{:?}", err)));
            }
        };
    }
    if let Err(commit_err) = tx.commit().await {
        return Json(ApiResponse::err(&format!("事务提交失败: {:?}", commit_err)));
    }
    return Json(ApiResponse::succ(Some("ok".to_string())));
}

// 角色取消绑定用户
pub async fn remove_user(
    Extension(curr_user): Extension<comm_api::CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<role_api::RoleAddUserReq>,
) -> Json<ApiResponse<String>> {
    if let Err(error) = req.validate() {
        return Json(ApiResponse::new(400, None, &format!("{}", error)));
    }
    let pool = db_pool();
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return Json(ApiResponse::err(&format!("开启事务失败:{:?}", err))),
    };
    for uid in req.userIds {
        let add_data = user_roles_role_model::UserRolesRole {
            userId: uid,
            roleId: id,
        };
        match user_roles_role_model::delete_user_roles_by_user_role_id(&mut tx, uid, id).await {
            Ok(_) => {}
            Err(err) => {
                if let Err(rollback_err) = tx.rollback().await {
                    return Json(ApiResponse::err(&format!(
                        "事务提交失败: {:?}",
                        rollback_err
                    )));
                }
                return Json(ApiResponse::err(&format!("删除用户角色失败:{:?}", err)));
            }
        };
    }
    if let Err(commit_err) = tx.commit().await {
        return Json(ApiResponse::err(&format!("事务提交失败: {:?}", commit_err)));
    }
    return Json(ApiResponse::succ(Some("ok".to_string())));
}

// 新增角色
pub async fn delete_role( Path(id): Path<i64>,) -> Json<ApiResponse<String>> {
    let pool = db_pool();
    // let pool = DB_POOL
    //     .lock()
    //     .unwrap()
    //     .as_ref()
    //     .expect("DB pool not initialized")
    //     .clone();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return Json(ApiResponse::err(&format!("开启事务失败:{:?}", err))),
    };

    match role_model::delete_role_by_id(&mut tx, id).await {
        Ok(_) => {}
        Err(err) => {
            if let Err(rollback_err) = tx.rollback().await {
                return Json(ApiResponse::err(&format!(
                    "事务回滚失败: {:?}",
                    rollback_err
                )));
            }
            return Json(ApiResponse::err(&format!(
                "删除角色失败:{:?}",
                err
            )));
        }
    };
    // 删除角色-资源关系表
    match role_permissions_permission::delete_permissions_by_role_id(&mut tx, id).await {
        Ok(_) => {}
        Err(err) => {
            if let Err(rollback_err) = tx.rollback().await {
                return Json(ApiResponse::err(&format!(
                    "事务回滚失败: {:?}",
                    rollback_err
                )));
            }
            return Json(ApiResponse::err(&format!(
                "删除角色-资源权限失败:{:?}",
                err
            )));
        }
    };
    if let Err(commit_err) = tx.commit().await {
        return Json(ApiResponse::err(&format!("事务提交失败: {:?}", commit_err)));
    }
    return Json(ApiResponse::succ(Some("ok".to_string())));
}
