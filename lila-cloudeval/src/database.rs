use std::{cmp::Reverse, collections::HashSet, path::PathBuf, sync::Arc};

use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{
    uci::UciMove,
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode, Position, Setup,
};
use terarkdb::{BlockBasedTableOptions, Cache, Db, Error as DbError, LogFile, Options};
use tokio::{task, task::JoinHandle};

use crate::{
    cdb_fen::cdb_fen,
    cdb_moves::{ScoredMoves, SortedScoredMoves},
};

#[derive(Debug, clap::Parser)]
pub struct DatabaseOpt {
    #[arg(long)]
    db_path: PathBuf,
    #[arg(long, default_value = "104857600")] // 100 MiB
    db_block_cache_bytes: usize,
}

impl DatabaseOpt {
    fn to_options(&self) -> Options {
        let mut options = Options::default();
        options
            .increase_parallelism(16)
            .set_block_based_table_options(
                BlockBasedTableOptions::default()
                    .set_block_cache(&Cache::new_lru(self.db_block_cache_bytes)),
            );
        options
    }
}

#[serde_as]
#[derive(Serialize)]
pub struct Pv {
    score: i16,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    line: Vec<UciMove>,
}

struct TiebrokenMove {
    uci: UciMove,
    score: i16,
    scored_child_moves: Option<ScoredMoves>,
}

impl TiebrokenMove {
    fn sort_key(&self) -> impl Ord + PartialOrd {
        (
            Reverse(self.score),
            self.scored_child_moves
                .as_ref()
                .map_or(0, |s| s.num_good_moves()),
        )
    }
}

#[derive(Debug)]
pub struct Database {
    inner: Db,
}

impl Database {
    pub fn open_blocking(opt: &DatabaseOpt) -> Result<Database, DbError> {
        Ok(Database {
            inner: Db::open(&opt.to_options(), &opt.db_path)?,
        })
    }

    pub fn open_read_only_blocking(opt: &DatabaseOpt) -> Result<Database, DbError> {
        Ok(Database {
            inner: Db::open_read_only(&opt.to_options(), &opt.db_path, LogFile::Ignore)?,
        })
    }

    pub async fn get_multi_pv(
        self: Arc<Self>,
        pos: Chess,
        multi_pv: usize,
    ) -> Result<Option<Vec<Pv>>, DbError> {
        let Some(root) = self.clone().multi_pv_root(pos.clone(), multi_pv).await? else {
            return Ok(None);
        };

        let db = &self;
        let extend_pv_handles: Vec<JoinHandle<_>> = root
            .into_iter()
            .map(move |begin| task::spawn(db.clone().extend_pv(pos.clone(), begin)))
            .collect();

        let mut result = Vec::with_capacity(extend_pv_handles.len());
        for handle in extend_pv_handles {
            result.push(handle.await.expect("join extend pv")?);
        }
        Ok(Some(result))
    }

    async fn multi_pv_root(
        self: Arc<Self>,
        pos: Chess,
        multi_pv: usize,
    ) -> Result<Option<Vec<TiebrokenMove>>, DbError> {
        task::spawn_blocking(move || self.multi_pv_root_blocking(&pos, multi_pv))
            .await
            .expect("multi pv root blocking")
    }

    fn multi_pv_root_blocking(
        &self,
        pos: &Chess,
        multi_pv: usize,
    ) -> Result<Option<Vec<TiebrokenMove>>, DbError> {
        let Some(root) = self.get_blocking(pos.clone().into_setup(EnPassantMode::Legal))? else {
            return Ok(None); // Root position not found
        };

        if root.len() < multi_pv && root.len() < pos.legal_moves().len() {
            return Ok(None); // Cannot satisfy number of requested pvs
        }

        let mut tiebroken_moves = self.tiebreak_moves_blocking(&pos, root, multi_pv)?;
        tiebroken_moves.sort_by_key(TiebrokenMove::sort_key);
        tiebroken_moves.truncate(multi_pv);
        Ok(Some(tiebroken_moves))
    }

    fn tiebreak_moves_blocking(
        &self,
        pos: &Chess,
        moves: SortedScoredMoves,
        at_least: usize,
    ) -> Result<Vec<TiebrokenMove>, DbError> {
        let best_moves = moves.into_best_moves(at_least);

        let (keys, natural_orders): (Vec<_>, Vec<_>) = best_moves
            .moves()
            .iter()
            .map(|entry| {
                let mut child = pos.clone();
                let m = entry.uci.to_move(&child).unwrap();
                child.play_unchecked(&m);
                cdb_fen(&child.into_setup(EnPassantMode::Legal))
            })
            .unzip();

        best_moves
            .into_moves()
            .into_iter()
            .zip(self.inner.multi_get(&keys).into_iter().zip(natural_orders))
            .map(|(entry, (row, natural_order))| {
                Ok(TiebrokenMove {
                    uci: entry.uci,
                    score: entry.score,
                    scored_child_moves: match row {
                        Ok(Some(value)) => {
                            Some(ScoredMoves::read_cdb(&mut &value[..], natural_order))
                        }
                        Ok(None) => None,
                        Err(err) => return Err(err),
                    },
                })
            })
            .collect::<Result<_, _>>()
    }

    fn get_blocking(&self, setup: Setup) -> Result<Option<SortedScoredMoves>, DbError> {
        let (key, natural_order) = cdb_fen(&setup);

        Ok(self
            .inner
            .get_pinned(key.as_bytes())?
            .map(|value| ScoredMoves::read_cdb(&mut &value[..], natural_order).into_sorted()))
    }

    async fn extend_pv(self: Arc<Self>, pos: Chess, begin: TiebrokenMove) -> Result<Pv, DbError> {
        task::spawn_blocking(move || self.extend_pv_blocking(pos, begin))
            .await
            .expect("extend pv blocking")
    }

    fn extend_pv_blocking(&self, mut pos: Chess, begin: TiebrokenMove) -> Result<Pv, DbError> {
        let score = begin.score;
        let mut line = vec![];
        let mut seen_hashes: HashSet<Zobrist64> = HashSet::new();
        let mut maybe_top_move = Some(begin);

        loop {
            //if pv.len() >= max_length {
            //    break;
            //}

            if !seen_hashes.insert(pos.zobrist_hash(EnPassantMode::Legal)) {
                break;
            }

            let Some(top_move) = maybe_top_move else {
                break;
            };

            let m = top_move.uci.to_move(&pos).expect("top move is legal");
            line.push(UciMove::from_chess960(&m));
            pos.play_unchecked(&m);

            let Some(scored_moves) = top_move.scored_child_moves else {
                break;
            };

            maybe_top_move = self
                .tiebreak_moves_blocking(&pos, scored_moves.into_sorted(), 1)?
                .into_iter()
                .min_by_key(TiebrokenMove::sort_key);
        }

        Ok(Pv { line, score })
    }
}
