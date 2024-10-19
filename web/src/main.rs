use crate::config::Config;
use crate::docs::{doc_add_prefix, ApiDoc};
use actix_files as af;
use actix_web::dev::ServiceRequest;
use actix_web::{
    get,
    http::header::ContentType,
    middleware,
    web::{scope, Data, ServiceConfig},
    App, HttpResponse, HttpServer, Responder,
};
use models::site::Site;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteJournalMode;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::str::FromStr;
use tokio::sync::Mutex;
use utoipa::OpenApi;

mod admin;
mod api;
mod config;
mod docs;
mod models;
mod utils;

pub struct AppState {
    pub sql: Pool<Sqlite>,
    pub sites: Mutex<HashMap<i64, Site>>,
}

#[get("/openapi.json")]
async fn openapi() -> impl Responder {
    let mut doc = ApiDoc::openapi();
    doc.merge(api::user::ApiDoc::openapi());
    doc.merge(api::verification::ApiDoc::openapi());
    doc.merge(api::sites::ApiDoc::openapi());

    let mut admin_doc = ApiDoc::openapi();
    admin_doc.merge(admin::sites::ApiDoc::openapi());

    doc_add_prefix(&mut admin_doc, "/admin", false);
    doc.merge(admin_doc);
    doc_add_prefix(&mut doc, "/api", false);
    HttpResponse::Ok().json(doc)
}

#[get("/rapidoc")]
async fn rapidoc() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body(
        r###"<!doctype html>
    <html><head><meta charset="utf-8"><style>rapi-doc {
    --green: #00dc7d; --blue: #5199ff; --orange: #ff6b00;
    --red: #ec0f0f; --yellow: #ffd600; --purple: #782fef; }</style>
    <script type="module" src="/static/rapidoc.js"></script></head><body> <rapi-doc spec-url="/openapi.json" persist-auth="true"
    bg-color="#040404" text-color="#f2f2f2"
    header-color="#040404" primary-color="#ec0f0f"
    nav-text-color="#eee" font-size="largest"
    allow-spec-url-load="false" allow-spec-file-load="false"
    show-method-in-nav-bar="as-colored-block" response-area-height="500px"
    show-header="false" schema-expand-level="1" /></body> </html>"###,
    )
}

fn config_app(app: &mut ServiceConfig) {
    if cfg!(debug_assertions) {
        app.service(af::Files::new("/static", "static"));
        app.service(af::Files::new("/assets", "dist/assets"));
        app.service(af::Files::new("/record", Config::RECORD_DIR));
    }

    app.service(openapi).service(rapidoc);
    app.service(
        scope("/api")
            .service(api::user::router())
            .service(api::verification::verification)
            .service(api::sites::router())
            .service(scope("/admin").service(admin::sites::router())),
    );
    app.default_service(|r: ServiceRequest| {
        actix_utils::future::ok(
            r.into_response(
                HttpResponse::Ok().content_type(ContentType::html()).body(
                    read_to_string("dist/index.html")
                        .unwrap_or("no index.html".to_string()),
                ),
            ),
        )
    });
}

async fn init() -> SqlitePool {
    dotenvy::from_path(".env").expect("could not read .env file");
    pretty_env_logger::init();

    let _ = std::fs::create_dir(Config::RECORD_DIR);
    let cpt = SqliteConnectOptions::from_str("sqlite://main.db")
        .expect("could not init sqlite connection options")
        .journal_mode(SqliteJournalMode::Off);

    SqlitePool::connect_with(cpt).await.expect("sqlite connection")
    // let pool = SqlitePool::connect("sqlite://main.db").await.unwrap();
}

#[cfg(unix)]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = init().await;

    let sites = sqlx::query_as! {
        Site,
        "select * from sites"
    }
    .fetch_all(&pool)
    .await
    .expect("could not get the sites")
    .iter()
    .map(|s| (s.id, s.clone()))
    .collect::<HashMap<_, _>>();

    let data = Data::new(AppState { sql: pool, sites: Mutex::new(sites) });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("%s %r %Ts"))
            .app_data(data.clone())
            .configure(config_app)
    });

    let server = if cfg!(debug_assertions) {
        server.bind(("127.0.0.1", 7000)).unwrap()
    } else {
        use std::os::unix::fs::PermissionsExt;
        const PATH: &str = "/usr/share/nginx/socks/heimdall.web.sock";
        let server = server.bind_uds(PATH).unwrap();
        std::fs::set_permissions(PATH, std::fs::Permissions::from_mode(0o777))?;
        server
    };

    server.run().await
}
