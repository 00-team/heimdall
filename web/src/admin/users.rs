use actix_web::web::{Data, Json};
use actix_web::{post, Scope};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use crate::docs::UpdatePaths;
use crate::models::user::{Admin, User};
use crate::models::Response;
use crate::{utils, AppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin::users")),
    paths(add),
    components(schemas(AdminUsersAddBody)),
    servers((url = "/users")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct AdminUsersAddBody {
    phone: String,
    name: String,
}

#[utoipa::path(
    post,
    request_body = AdminUsersAddBody,
    responses((status = 200, body = User))
)]
/// Add
#[post("/")]
async fn add(_: Admin, body: Json<AdminUsersAddBody>, state: Data<AppState>) -> Response<User> {
    utils::phone_validator(&body.phone)?;

    let mut user = User {
        id: 0,
        phone: body.phone.clone(),
        name: body.name.clone(),
        token: None,
        admin: false,
    };

    let result = sqlx::query! {
        "insert into users(phone, name) values(?, ?)",
        user.phone, user.name
    }
    .execute(&state.sql)
    .await?;

    user.id = result.last_insert_rowid();

    Ok(Json(user))
}

pub fn router() -> Scope {
    Scope::new("/users").service(add)
}
