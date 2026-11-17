use salvo::prelude::*;
use salvo::session::{SessionDepotExt, SessionHandler, MemoryStore};
use crate::models::ApiResponse;
use crate::state::AppState;
use serde_json::json;

pub const ADMIN_SESSION_KEY: &str = "admin_user_id";
pub const ADMIN_USERNAME_KEY: &str = "admin_username";

#[handler]
pub async fn auth_middleware(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let user_id = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_SESSION_KEY));

    if let Some(user_id) = user_id {
        depot.insert("admin_user_id", user_id);
        ctrl.call_next(req, depot, res).await;
    } else {
        let path = req.uri().path();
        if path.starts_with("/api/admin") || path.starts_with("/admin/api") {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(json!({
                "success": false,
                "message": "Unauthorized: Please login first",
                "redirect": "/admin/login"
            })));
        } else {
            res.render(Redirect::found("/admin/login"));
        }
    }
}

#[handler]
pub async fn optional_auth(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    if let Some(user_id) = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_SESSION_KEY))
    {
        depot.insert("admin_user_id", user_id);
    }
    ctrl.call_next(req, depot, res).await;
}

#[handler]
pub async fn login_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let form = req.parse_form::<crate::models::LoginForm>().await.unwrap_or_default();

    let user = sqlx::query_as::<_, crate::models::AdminUser>(
        "SELECT * FROM admin_users WHERE username = ? AND is_active = 1"
    )
    .bind(&form.username)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some(user) = user {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let parsed_hash = PasswordHash::new(&user.password_hash).ok();
        let is_valid = parsed_hash
            .and_then(|hash| Argon2::default().verify_password(form.password.as_bytes(), &hash).ok())
            .is_some();

        if is_valid {
            sqlx::query("UPDATE admin_users SET last_login = datetime('now') WHERE id = ?")
                .bind(&user.id)
                .execute(&state.db)
                .await
                .ok();

            if let Some(session) = depot.session_mut() {
                session.insert(ADMIN_SESSION_KEY, user.id.to_string()).ok();
                session.insert(ADMIN_USERNAME_KEY, user.username.clone()).ok();
            }

            res.render(Json(ApiResponse::ok(json!({
                "redirect": "/admin/dashboard",
                "user": { "username": user.username, "name": user.full_name }
            }))));
            return;
        }
    }

    res.status_code(StatusCode::BAD_REQUEST);
    res.render(Json(ApiResponse::<()>::err(
        "Username atau password salah".to_string(),
        None
    )));
}

#[handler]
pub async fn logout_handler(depot: &mut Depot, res: &mut Response) {
    if let Some(session) = depot.session_mut() {
        session.destroy();
    }
    res.render(Redirect::found("/admin/login"));
}

#[handler]
pub async fn check_auth_handler(depot: &mut Depot, res: &mut Response) {
    if let Ok(user_id) = depot.get::<String>("admin_user_id") {
        res.render(Json(ApiResponse::ok(json!({
            "authenticated": true,
            "user_id": user_id
        }))));
    } else {
        res.render(Json(ApiResponse::ok(json!({
            "authenticated": false
        }))));
    }
}
