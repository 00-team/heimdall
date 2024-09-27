use crate::api::verification;
use crate::config::Config;
use crate::docs::UpdatePaths;
use crate::models::user::User;
use crate::models::{AppErr, Response};
use crate::utils::{self, CutOff};
use crate::AppState;

use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::web::{Data, Json};
use actix_web::{get, patch, post, HttpResponse, Scope};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    tags((name = "api::user")),
    paths(login, logout, get, update),
    components(schemas(User, LoginBody, UserUpdateBody)),
    servers((url = "/user")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Debug, Deserialize, ToSchema)]
struct LoginBody {
    phone: String,
    code: String,
}

#[utoipa::path(
    post,
    request_body = LoginBody,
    responses((status = 200, body = User))
)]
/// Login
#[post("/login/")]
async fn login(
    body: Json<LoginBody>, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    verification::verify(&body.phone, &body.code, verification::Action::Login)
        .await?;

    let token = utils::get_random_string(Config::TOKEN_ABC, 69);

    let result = sqlx::query_as! {
        User,
        "select * from users where phone = ?",
        body.phone
    }
    .fetch_one(&state.sql)
    .await;

    let user: User = match result {
        Ok(mut user) => {
            user.token = Some(token.clone());

            let _ = sqlx::query_as! {
                User,
                "update users set token = ? where id = ?",
                token, user.id
            }
            .execute(&state.sql)
            .await;

            user
        }
        Err(_) => {
            let result = sqlx::query_as! {
                User,
                "insert into users (phone, token) values(?, ?)",
                body.phone, token
            }
            .execute(&state.sql)
            .await;

            User {
                id: result.unwrap().last_insert_rowid(),
                phone: body.phone.clone(),
                token: Some(token.clone()),
                ..Default::default()
            }
        }
    };

    let cook =
        Cookie::build("authorization", format!("user {}:{token}", user.id))
            .path("/")
            .secure(true)
            .same_site(SameSite::Lax)
            .http_only(true)
            .max_age(Duration::weeks(12))
            .finish();

    Ok(HttpResponse::Ok().cookie(cook).json(user))
}

#[utoipa::path(post, responses((status = 200)))]
#[post("/logout/")]
/// Logout
async fn logout(user: User, state: Data<AppState>) -> HttpResponse {
    let _ = sqlx::query! {
        "update users set token = null where id = ?",
        user.id
    }
    .execute(&state.sql)
    .await;

    let cook = Cookie::build("authorization", "")
        .path("/")
        .secure(true)
        .same_site(SameSite::Lax)
        .http_only(true)
        .max_age(Duration::seconds(1))
        .finish();

    HttpResponse::Ok().cookie(cook).finish()
}

#[utoipa::path(get, responses((status = 200, body = User)))]
#[get("/")]
/// Get
async fn get(user: User) -> Json<User> {
    Json(user)
}

#[derive(Deserialize, ToSchema)]
struct UserUpdateBody {
    name: String,
}

#[utoipa::path(
    patch,
    request_body = UserUpdateBody,
    responses((status = 200, body = User))
)]
/// Update
#[patch("/")]
async fn update(
    user: User, body: Json<UserUpdateBody>, state: Data<AppState>,
) -> Response<User> {
    let mut user = user;
    user.name = body.name.clone();
    user.name.cut_off(255);

    sqlx::query_as! {
        User,
        "update users set name = ? where id = ?",
        user.name, user.id
    }
    .execute(&state.sql)
    .await?;

    Ok(Json(user))
}

pub fn router() -> Scope {
    Scope::new("/user")
        .service(login)
        .service(logout)
        .service(get)
        .service(update)
}
