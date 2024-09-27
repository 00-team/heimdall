use actix_web::web::{Data, Json};
use actix_web::{post, Scope};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use crate::docs::UpdatePaths;
use crate::models::user::Admin;
use crate::models::{site::Site, Response};
use crate::AppState;

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin::sites")),
    paths(add),
    components(schemas(Site, SitesAddBody)),
    servers((url = "/sites")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Deserialize, ToSchema)]
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

    let result = sqlx::query! {
        "insert into sites(name) values(?)",
        body.name
    }
    .execute(&state.sql)
    .await?;

    let site = Site {
        id: result.last_insert_rowid(),
        name: body.name.clone(),
        ..Default::default()
    };
    let sites = state.sites.lock()?;
    sites.insert(site.id, site);

    Ok(Json(site))
}

pub fn router() -> Scope {
    Scope::new("/sites").service(add)
}
