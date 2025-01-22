use actix_web::web::{Data, Json, Path, Query};
use actix_web::{get, post, FromRequest, HttpRequest, HttpResponse, Scope};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use crate::config::config;
use crate::docs::UpdatePaths;
use crate::models::deploy::{Deploy, DeployStatus};
use crate::models::user::User;
use crate::models::{bad_request, not_found, AppErr, ListInput};
use crate::models::{forbidden, Response};
use crate::{utils, AppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "api::deploy")),
    paths(list, add),
    components(schemas(
        Deploy, DeployStatus
    )),
    servers((url = "/deploy")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    params(ListInput),
    responses((status = 200, body = Vec<Deploy>))
)]
/// List
#[get("/")]
async fn list(
    _: User, q: Query<ListInput>, state: Data<AppState>,
) -> Response<Vec<Deploy>> {
    let offset = q.page * 32;
    let deploy = sqlx::query_as! {
        Deploy,
        "select * from deploys limit 32 offset ?",
        offset
    }
    .fetch_all(&state.sql)
    .await?;

    Ok(Json(deploy))
}

#[derive(Deserialize, ToSchema)]
struct GithubPushEventAuthor {
    name: String,
}

#[derive(Deserialize, ToSchema)]
struct GithubPushEventCommit {
    message: String,
    author: GithubPushEventAuthor,
}

#[derive(Deserialize, ToSchema)]
struct GithubPushEvent {
    commits: Vec<GithubPushEventCommit>,
}

#[utoipa::path(
    post,
    params(
        ("repo" = String, Path, example = "simurgh"),
        ("actor" = String, Path, example = "github"),
        ("pass" = String, Path, example = "<password>"),
    ),
    responses((status = 200))
)]
/// Add
#[post("/{repo}/{actor}/{pass}/")]
async fn add(
    rq: HttpRequest, path: Path<(String, String, String)>,
    state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let (repo, actor, pass) = path.into_inner();

    let path = config().deploy_repo.get(repo.as_str());

    let Some(path) = path else {
        return Err(not_found!("repo not found"));
    };

    if config().deploy_key != pass {
        return Err(forbidden!("invalid password"));
    }

    let now = utils::now();
    let sender = match actor.as_str() {
        "github" => {
            if !matches!(
                rq.headers().get("X-GitHub-Event").map(|v| v.to_str()),
                Some(Ok("push"))
            ) {
                return Err(bad_request!("bad github event. only push"));
            }

            let Ok(event) = Json::<GithubPushEvent>::extract(&rq).await else {
                return Err(bad_request!("invalid body"));
            };

            let Some(commit) = event
                .commits
                .iter()
                .rev()
                .find(|c| c.message.contains("release"))
            else {
                return Ok(HttpResponse::Ok().finish());
            };

            commit.author.name.clone()
        }
        "sadra" | "007" => actor.clone(),
        _ => return Err(forbidden!("unknown actor")),
    };

    let id = sqlx::query! {
        "insert into deploys(repo, actor, sender, begin, status) values(?,?,?,?,?)",
        repo, actor, sender, now, DeployStatus::Pending
    }
    .execute(&state.sql)
    .await?
    .last_insert_rowid();

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/deploy").service(list).service(add)
}
