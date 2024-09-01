use shakmaty::Setup;
use std::path::PathBuf;
use terarkdb::BlockBasedTableOptions;
use terarkdb::Cache;
use terarkdb::Db;
use terarkdb::Error as DbError;
use terarkdb::LogFile;
use terarkdb::Options;

use crate::cdb_fen::cdb_fen;
use crate::cdb_moves::ScoredMoves;

pub struct DatabaseOpt {
    db_path: PathBuf,
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

pub struct Database {
    inner: Db,
}

impl Database {
    pub fn open_blocking(opt: &DatabaseOpt) -> Result<Database, DbError> {
        Ok(Database {
            inner: Db::open(&opt.to_options(), &opt.db_path)?,
        })
    }

    pub fn open_for_readonly_blocking(opt: &DatabaseOpt) -> Result<Database, DbError> {
        Ok(Database {
            inner: Db::open_read_only(&opt.to_options(), &opt.db_path, LogFile::Ignore)?,
        })
    }

    pub fn get_blocking(&self, setup: &Setup) -> Result<Option<ScoredMoves>, DbError> {
        let bin_fen = cdb_fen(setup);
        let bin_fen_mirrored = cdb_fen(&setup.clone().into_mirrored());
        let natural_order = bin_fen.as_bytes() < bin_fen_mirrored.as_bytes();

        let maybe_value = self.inner.get(match natural_order {
            false => bin_fen.as_bytes(),
            true => bin_fen_mirrored.as_bytes(),
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
