use actix_web::web::{self, Data, Json, Query};
use actix_web::{get, post, rt, HttpRequest, HttpResponse, Scope};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use crate::docs::UpdatePaths;
use crate::models::user::{Authorization, User};
use crate::models::{site::Site, Response};
use crate::models::{AppErr, AppErrNotFound, ListInput};
use crate::{utils, AppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "api::sites")),
    paths(list, dump, ping, ws_test),
    components(schemas(Site, SiteDumpBody)),
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

#[derive(Deserialize, ToSchema)]
struct SiteDumpBody {
    total: i64,
    total_time: i64,
}

#[utoipa::path(
    post,
    request_body = SiteDumpBody,
    responses((status = 200))
)]
/// Dump
#[post("/dump/")]
async fn dump(
    rq: HttpRequest, body: Json<SiteDumpBody>, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let mut sites = state.sites.lock().await;
    let site = match Authorization::try_from(&rq)? {
        Authorization::Site { id, token } => sites
            .get_mut(&id)
            .and_then(|v| if v.token == Some(token) { Some(v) } else { None })
            .and_then(|v| if v.online { Some(v) } else { None })
            .ok_or(()),
        _ => Err(()),
    }
    .map_err(|_| AppErrNotFound("no site was found"))?;

    site.total_requests += body.total;
    site.total_requests_time += body.total_time;
    site.latest_request = utils::now();

    sqlx::query! {"
        update sites set
        total_requests = ?,
        total_requests_time = ?,
        latest_request = ?
        where id = ?
    ",
        site.total_requests,
        site.total_requests_time,
        site.latest_request,
        site.id
    }
    .execute(&state.sql)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    post,
    responses((status = 200))
)]
/// Ping
#[post("/ping/")]
async fn ping(
    rq: HttpRequest, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let mut sites = state.sites.lock().await;
    let site = match Authorization::try_from(&rq)? {
        Authorization::Site { id, token } => sites
            .get_mut(&id)
            .and_then(|v| if v.token == Some(token) { Some(v) } else { None })
            .and_then(|v| if v.online { Some(v) } else { None })
            .ok_or(()),
        _ => Err(()),
    }
    .map_err(|_| AppErrNotFound("no site was found"))?;

    site.latest_ping = utils::now();

    sqlx::query! {
        "update sites set latest_ping = ? where id = ?",
        site.latest_ping, site.id
    }
    .execute(&state.sql)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(get)]
/// WebSocks Test
#[get("/ws-test/")]
async fn ws_test(
    rq: HttpRequest, stream: web::Payload, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let (res, mut session, stream) = actix_ws::handle(&rq, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(1024 * 1024);

    // let _ = session.text("welcome kid").await;

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            if let Err(e) = &msg {
                log::info!("ws msg err: {e:?}");
            }

            log::info!("ws msg: {msg:?}");
            match msg.unwrap() {
                // AggregatedMessage::Text() => {
                //     for (_, site) in sites.iter() {
                //         let _ = session
                //             .text(serde_json::to_string(site).expect("xxx"))
                //             .await;
                //     }
                //     // let res = session.text("a message from server").await;
                //     // log::info!("res: {res:?}");
                //     // let res = session.ping(b"").await;
                //     // log::info!("ping res: {res:?}");
                // }
                AggregatedMessage::Text(txt) => {
                    let id = txt.parse::<i64>();
                    if id.is_err() {
                        return;
                    }
                    let sites = state.sites.lock().await;
                    let site = sites.get(&id.unwrap());
                    if site.is_none() {
                        let _ = session
                            .text(
                                serde_json::to_string(&AppErrNotFound(
                                    "site not found",
                                ))
                                .unwrap(),
                            )
                            .await;
                        return;
                    }
                    let _ = session
                        .text(serde_json::to_string(site.unwrap()).unwrap())
                        .await;
                }
                AggregatedMessage::Ping(bytes) => {
                    let res = session.pong(&bytes).await;
                    log::info!("pong res: {res:?}");
                }
                _ => {} // AggregatedMessage::Pong(_) => {}
                        // AggregatedMessage::Close(_) => {}
            }
        }
    });

    Ok(res)
}

pub fn router() -> Scope {
    Scope::new("/sites")
        .service(list)
        .service(dump)
        .service(ping)
        .service(ws_test)
}
