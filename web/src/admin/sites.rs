use actix_web::web::{Data, Json};
use actix_web::{post, Scope};
use serde::Deserialize;
use utoipa::OpenApi;

use crate::docs::UpdatePaths;
use crate::models::user::Admin;
use crate::models::{site::Site, Response};
use crate::AppState;

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin::sites")),
    paths(add),
    components(schemas(Site)),
    servers((url = "/sites")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Deserialize)]
struct SitesAddBody {
    name: String,
}

#[utoipa::path(
    post,
    request_body = SitesAddBody,
    responses((status = 200, body = Site))
)]
/// Add
#[post("/")]
async fn add(
    _: Admin, body: Json<SitesAddBody>, state: Data<AppState>,
) -> Response<Site> {
    Site::verify_name(&body.name)?;
    let mut site = Site {
        id: 0,
        name: body.name.clone(),
        latest_request: 0,
        total_requests: 0,
        token: None,
    };
    let result = sqlx::query! {
        "insert into sites(name) values(?)",
        site.name
    }
    .execute(&state.sql)
    .await?;

    site.id = result.last_insert_rowid();

    Ok(Json(site))
}

pub fn router() -> Scope {
    Scope::new("/sites").service(add)
}
