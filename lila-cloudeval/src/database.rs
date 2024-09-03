use std::{collections::HashSet, path::PathBuf};

use shakmaty::{
    uci::UciMove,
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode, Position, Setup,
};
use terarkdb::{BlockBasedTableOptions, Cache, Db, Error as DbError, LogFile, Options};

use crate::cdb_moves::SortedScoredMoves;
use crate::{cdb_fen::cdb_fen, cdb_moves::ScoredMoves};

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

    pub fn get_blocking(&self, setup: Setup) -> Result<Option<SortedScoredMoves>, DbError> {
        let (key, natural_order) = cdb_fen(&setup);

        Ok(self
            .inner
            .get_pinned(key.as_bytes())?
            .map(|value| ScoredMoves::read_cdb(&mut &value[..], natural_order).into_sorted()))
    }

    pub fn get_pv_blocking(
        &self,
        mut pos: Chess,
        max_length: usize,
    ) -> Result<Vec<UciMove>, DbError> {
        let mut pv = Vec::new();

        let mut seen_hashes: HashSet<Zobrist64> = HashSet::new();

        let mut maybe_scored_moves =
            self.get_blocking(pos.clone().into_setup(EnPassantMode::Legal))?;

        loop {
            let Some(scored_moves) = maybe_scored_moves else {
                break;
            };

            if pv.len() >= max_length {
                break;
            }

            if !seen_hashes.insert(pos.zobrist_hash(EnPassantMode::Legal)) {
                break;
            }

            let best_moves = scored_moves.best_moves();

            let (keys, natural_orders): (Vec<_>, Vec<_>) = best_moves
                .iter()
                .map(|entry| {
                    let mut child = pos.clone();
                    let m = entry.uci.to_move(&child).unwrap();
                    child.play_unchecked(&m);
                    cdb_fen(&child.into_setup(EnPassantMode::Legal))
                })
                .unzip();

            let scored_child_moves: Vec<(Option<ScoredMoves>, UciMove)> = self
                .inner
                .multi_get(&keys)
                .into_iter()
                .zip(natural_orders)
                .zip(best_moves)
                .map(|((row, natural_order), entry)| match row {
                    Ok(Some(value)) => Ok((
                        Some(ScoredMoves::read_cdb(&mut &value[..], natural_order)),
                        entry.uci.clone(),
                    )),
                    Ok(None) => Ok((None, entry.uci.clone())),
                    Err(err) => Err(err),
                })
                .collect::<Result<_, _>>()?;

            let Some((maybe_top_scored_moves, top_uci)) = scored_child_moves
                .into_iter()
                .min_by_key(|(scored_moves, _)| {
                    scored_moves.as_ref().map_or(0, |s| s.num_good_moves())
                })
            else {
                break;
            };

            let m = top_uci.to_move(&pos).unwrap();
            pv.push(UciMove::from_chess960(&m));
            pos.play_unchecked(&m);
            maybe_scored_moves = maybe_top_scored_moves.map(ScoredMoves::into_sorted);
        }

        Ok(pv)
    }
}
