use std::{collections::HashSet, path::PathBuf};

use shakmaty::{
    uci::UciMove,
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode, Position, Setup,
};
use terarkdb::{BlockBasedTableOptions, Cache, Db, Error as DbError, LogFile, Options};

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

    pub fn get_blocking(&self, setup: Setup) -> Result<Option<ScoredMoves>, DbError> {
        let (key, natural_order) = cdb_fen(&setup);

        let Some(value) = self.inner.get_pinned(key.as_bytes())? else {
            return Ok(None);
        };

        let mut scored_moves = ScoredMoves::read_cdb(&mut &value[..], natural_order);
        scored_moves.sort_by_score();
        Ok(Some(scored_moves))
    }

    pub fn get_pv_blocking(
        &self,
        mut pos: Chess,
        max_length: usize,
    ) -> Result<Vec<UciMove>, DbError> {
        let mut pv = Vec::new();

        let mut seen_hashes: HashSet<Zobrist64> = HashSet::new();

        loop {
            if pv.len() >= max_length {
                break;
            }

            if !seen_hashes.insert(pos.zobrist_hash(EnPassantMode::Legal)) {
                break;
            }

            let Some(scored_moves) =
                self.get_blocking(pos.clone().into_setup(EnPassantMode::Legal))?
            else {
                break;
            };

            let best_moves = scored_moves.best_moves();
            if best_moves.is_empty() {
                break;
            }

            let top_move = if best_moves.len() == 1 {
                best_moves[0].uci.clone()
            } else {
                let (keys, natural_orders): (Vec<_>, Vec<_>) = best_moves
                    .iter()
                    .map(|entry| {
                        let mut child = pos.clone();
                        let m = entry.uci.to_move(&child).unwrap();
                        child.play_unchecked(&m);
                        cdb_fen(&child.into_setup(EnPassantMode::Legal))
                    })
                    .unzip();

                let values = self.inner.multi_get(&keys);

                best_moves
                    .into_iter()
                    .zip(values.into_iter().zip(natural_orders))
                    .min_by_key(|(_entry, (maybe_value, natural_order))| {
                        let Some(value) = maybe_value.as_ref().unwrap() else {
                            return 0;
                        };
                        let scored_child_moves =
                            ScoredMoves::read_cdb(&mut &value[..], *natural_order);
                        let n = scored_child_moves.num_good_moves();
                        println!("- {} ({})", _entry.uci, n);
                        n
                    })
                    .unwrap()
                    .0
                    .uci
                    .clone()
            };

            let m = top_move.to_move(&pos).unwrap();
            pv.push(UciMove::from_chess960(&m));
            pos.play_unchecked(&m);
        }

        Ok(pv)
    }
}
