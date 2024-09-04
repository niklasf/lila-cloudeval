use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{FromRef, Query, State},
    routing::{get, Router},
    Json,
};
use clap::Parser as _;
use lila_cloudeval::{
    database::{Database, DatabaseOpt, Pv},
    error::Error,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, TryFromInto};
use shakmaty::{fen::Fen, CastlingMode};
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

#[derive(Copy, Clone, Debug)]
struct MultiPv(usize);

impl Default for MultiPv {
    fn default() -> MultiPv {
        MultiPv(1)
    }
}

impl From<MultiPv> for usize {
    fn from(MultiPv(n): MultiPv) -> usize {
        n
    }
}

impl TryFrom<usize> for MultiPv {
    type Error = Error;

    fn try_from(n: usize) -> Result<MultiPv, Error> {
        if n > 5 {
            Err(Error::MultiPvRange { n })
        } else {
            Ok(MultiPv(n))
        }
    }
}

#[serde_as]
#[derive(Deserialize)]
struct PvQuery {
    #[serde_as(as = "DisplayFromStr")]
    fen: Fen,
    #[serde_as(as = "TryFromInto<usize>")]
    #[serde(default)]
    multi_pv: MultiPv,
}

#[derive(Serialize)]
struct PvResponse {
    pvs: Option<Vec<Pv>>,
}

#[axum::debug_handler(state = AppState)]
async fn query_pv(
    State(db): State<Arc<Database>>,
    Query(pv_query): Query<PvQuery>,
) -> Result<Json<PvResponse>, Error> {
    Ok(Json(PvResponse {
        pvs: db
            .get_multi_pv(
                pv_query.fen.into_position(CastlingMode::Chess960)?,
                pv_query.multi_pv.into(),
            )
            .await?,
    }))
}

// In: {"t":"evalGet","d":{"fen":"r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3","path":"/?WG)8\\M(D","mpv":2}}
// Out: {"t":"evalHit","d":{"fen":"r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3","knodes":7298073,"depth":51,"pvs":[{"moves":"g8f6 d2d3 f8c5 b5a4 d7d6 c2c3 e8h8 e1h1 c5b6 b1d2","cp":13},{"moves":"a7a6 b5a4 g8f6 e1h1 f8e7 f1e1 b7b5 a4b3 e8h8 a2a4","cp":20}],"path":"/?WG)8\\M(D"}}
