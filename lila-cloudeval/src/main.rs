use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{FromRef, Query, State},
    routing::{get, Router},
    Json,
};
use clap::Parser as _;
use lila_cloudeval::{
    database::{Database, DatabaseOpt},
    error::Error,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, uci::UciMove, CastlingMode};
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

#[serde_as]
#[derive(Serialize)]
struct PvResponse {
    #[serde_as(as = "Vec<Vec<DisplayFromStr>>")]
    pvs: Vec<Vec<UciMove>>,
}

#[axum::debug_handler(state = AppState)]
async fn query_pv(
    State(db): State<Arc<Database>>,
    Query(pv_query): Query<PvQuery>,
) -> Result<Json<PvResponse>, Error> {
    Ok(Json(PvResponse {
        pvs: vec![db.get_pv_blocking(pv_query.fen.into_position(CastlingMode::Chess960)?)?],
    }))
}
