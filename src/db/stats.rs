use anyhow::{anyhow, Result};
use heed::{
    types::{Bytes, Str},
    Database, Env,
};

/// Basic environment statistics.
#[derive(Debug, Clone)]
pub struct EnvStats {
    pub map_size: usize,
    pub last_page_number: usize,
    pub last_txn_id: usize,
    pub max_readers: u32,
    pub num_readers: u32,
}

/// Database statistics derived from LMDB.
#[derive(Debug, Clone)]
pub struct DbStats {
    pub page_size: u32,
    pub depth: u32,
    pub branch_pages: usize,
    pub leaf_pages: usize,
    pub overflow_pages: usize,
    pub entries: usize,
}

/// Gather environment statistics.
pub fn env_stats(env: &Env) -> EnvStats {
    let info = env.info();
    EnvStats {
        map_size: info.map_size,
        last_page_number: info.last_page_number,
        last_txn_id: info.last_txn_id,
        max_readers: info.maximum_number_of_readers,
        num_readers: info.number_of_readers,
    }
}

/// Gather statistics for a specific database by name.
pub fn db_stats(env: &Env, db_name: &str) -> Result<DbStats> {
    let rtxn = env.read_txn()?;
    let db: Database<Str, Bytes> = if db_name == "(unnamed)" {
        // Open the unnamed database
        env.open_database(&rtxn, None)?
            .ok_or_else(|| anyhow!("unnamed database not found"))?
    } else {
        // Open a named database
        env.open_database(&rtxn, Some(db_name))?
            .ok_or_else(|| anyhow!("database not found"))?
    };
    let stat = db.stat(&rtxn)?;
    rtxn.commit()?;
    Ok(DbStats {
        page_size: stat.page_size,
        depth: stat.depth,
        branch_pages: stat.branch_pages,
        leaf_pages: stat.leaf_pages,
        overflow_pages: stat.overflow_pages,
        entries: stat.entries,
    })
}
