use anyhow::Result;
use heed::{Env, RwTxn};

/// Wrapper around `heed::RwTxn` with simpler lifetime management.
pub struct Txn<'env> {
    inner: RwTxn<'env>,
}

impl<'env> Txn<'env> {
    /// Begin a new read-write transaction.
    pub fn begin(env: &'env Env) -> Result<Self> {
        let inner = env.write_txn()?;
        Ok(Self { inner })
    }

    /// Commit the transaction.
    pub fn commit(self) -> Result<()> {
        self.inner.commit()?;
        Ok(())
    }

    /// Abort the transaction.
    pub fn abort(self) {
        self.inner.abort();
    }

    pub(crate) fn inner(&self) -> &RwTxn<'env> {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut RwTxn<'env> {
        &mut self.inner
    }
}
