use std::collections::{HashSet, VecDeque};

/// Stores bookmarked keys per database.
#[derive(Default)]
pub struct Bookmarks {
    items: HashSet<(String, String)>,
}

impl Bookmarks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, db: String, key: String) {
        self.items.insert((db, key));
    }

    pub fn remove(&mut self, db: &str, key: &str) {
        self.items.remove(&(db.to_string(), key.to_string()));
    }

    pub fn contains(&self, db: &str, key: &str) -> bool {
        self.items.contains(&(db.to_string(), key.to_string()))
    }
}

/// Maintains a bounded jump-to-key history.
pub struct JumpHistory {
    list: VecDeque<(String, String)>,
    max: usize,
}

impl JumpHistory {
    pub fn new(max: usize) -> Self {
        Self {
            list: VecDeque::new(),
            max,
        }
    }

    pub fn push(&mut self, db: String, key: String) {
        if self.list.len() == self.max {
            self.list.pop_front();
        }
        self.list.push_back((db, key));
    }

    pub fn entries(&self) -> impl Iterator<Item = &(String, String)> {
        self.list.iter()
    }
}
