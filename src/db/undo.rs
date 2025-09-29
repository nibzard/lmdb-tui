use anyhow::Result;

use heed::Env;

use super::{kv, txn::Txn};

/// Represents a CRUD operation for undo/redo.
#[derive(Clone)]
pub enum Op {
    Put {
        db: String,
        key: String,
        prev: Option<Vec<u8>>,
        new: Vec<u8>,
    },
    Delete {
        db: String,
        key: String,
        prev: Option<Vec<u8>>,
    },
}

/// Simple undo/redo stack.
pub struct UndoStack {
    ops: Vec<Op>,
    pos: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            pos: 0,
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.pos > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.pos < self.ops.len()
    }

    /// Get number of operations that can be undone
    pub fn undo_count(&self) -> usize {
        self.pos
    }

    /// Get number of operations that can be redone
    pub fn redo_count(&self) -> usize {
        self.ops.len() - self.pos
    }

    pub fn push(&mut self, op: Op) {
        if self.pos < self.ops.len() {
            self.ops.truncate(self.pos);
        }
        self.ops.push(op);
        self.pos = self.ops.len();
    }

    pub fn undo(&mut self, env: &Env, txn: &mut Txn<'_>) -> Result<bool> {
        if self.pos == 0 {
            return Ok(false);
        }
        self.pos -= 1;
        let op = self.ops[self.pos].clone();
        match op {
            Op::Put { db, key, prev, .. } => match prev {
                Some(v) => kv::put(env, txn, &db, &key, &v)?,
                None => kv::delete(env, txn, &db, &key)?,
            },
            Op::Delete { db, key, prev } => {
                if let Some(v) = prev {
                    kv::put(env, txn, &db, &key, &v)?;
                }
            }
        }
        Ok(true)
    }

    pub fn redo(&mut self, env: &Env, txn: &mut Txn<'_>) -> Result<bool> {
        if self.pos == self.ops.len() {
            return Ok(false);
        }
        let op = self.ops[self.pos].clone();
        self.pos += 1;
        match op {
            Op::Put { db, key, new, .. } => kv::put(env, txn, &db, &key, &new)?,
            Op::Delete { db, key, .. } => kv::delete(env, txn, &db, &key)?,
        }
        Ok(true)
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}
