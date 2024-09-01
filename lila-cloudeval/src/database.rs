use std::path::PathBuf;

use shakmaty::Setup;
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

    pub fn get_blocking(&self, setup: &Setup) -> Result<Option<ScoredMoves>, DbError> {
        let bin_fen = cdb_fen(setup);
        let bin_fen_mirrored = cdb_fen(&setup.clone().into_mirrored());
        let natural_order = bin_fen.as_bytes() < bin_fen_mirrored.as_bytes();

        let maybe_value = self.inner.get(match natural_order {
            true => bin_fen.as_bytes(),
            false => bin_fen_mirrored.as_bytes(),
        })?;

        let Some(value) = maybe_value else {
            return Ok(None);
        };

        let mut scored_moves = ScoredMoves::read_cdb(&mut &value[..]);

        if !natural_order {
            scored_moves.mirror();
        }

        scored_moves.sort_by_score();

        Ok(Some(scored_moves))
    }
}
