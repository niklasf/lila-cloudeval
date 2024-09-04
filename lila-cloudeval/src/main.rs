use axum::extract::FromRef;
use axum::extract::Query;
use axum::extract::State;
use axum::routing::get;
use axum::routing::Router;
use axum::Json;
use clap::Parser as _;
use lila_cloudeval::database::Database;
use lila_cloudeval::database::DatabaseOpt;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::fen::Fen;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug, clap::Parser)]
struct Opt {
    #[clap(flatten)]
    db: DatabaseOpt,
    #[arg(long)]
    bind: SocketAddr,
}

#[derive(FromRef, Clone)]
struct AppState {
    db: Arc<Database>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    let app = Router::new()
        .route("/", get(query_pv))
        .with_state(AppState {
            db: Arc::new(Database::open_read_only_blocking(&opt.db).expect("open database")),
        });

    let listener = TcpListener::bind(&opt.bind).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

#[serde_as]
#[derive(Deserialize)]
struct PvQuery {
    #[serde_as(as = "DisplayFromStr")]
    fen: Fen,
}

#[derive(Serialize)]
struct PvResponse {}

#[axum::debug_handler(state = AppState)]
async fn query_pv(
    State(db): State<Arc<Database>>,
    Query(pv_query): Query<PvQuery>,
) -> Json<PvResponse> {
    Json(PvResponse {})
}
