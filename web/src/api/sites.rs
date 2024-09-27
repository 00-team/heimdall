use actix_web::web::{Data, Json, Query};
use actix_web::{get, Scope};
use utoipa::OpenApi;

use crate::docs::UpdatePaths;
use crate::models::user::User;
use crate::models::ListInput;
use crate::models::{site::Site, Response};
use crate::AppState;

#[derive(OpenApi)]
#[openapi(
    tags((name = "api::sites")),
    paths(list),
    components(schemas(Site)),
    servers((url = "/sites")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    params(ListInput),
    responses((status = 200, body = Vec<Site>))
)]
/// List
#[get("/")]
async fn list(
    _: User, q: Query<ListInput>, state: Data<AppState>,
) -> Response<Vec<Site>> {
    let offset = q.page * 32;
    let sites = sqlx::query_as! {
        Site,
        "select * from sites limit 32 offset ?",
        offset
    }
    .fetch_all(&state.sql)
    .await?;

    Ok(Json(sites))
}

pub fn router() -> Scope {
    Scope::new("/sites").service(list)
}
