use std::ops::{BitOr, Deref};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WordId(u32);

impl WordId {
    pub fn from_index(n: usize) -> Self {
        Self(1_u32 << n)
    }

    pub fn to_index(&self) -> usize {
        self.0.trailing_zeros() as usize
    }
}

impl Deref for WordId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BitOr for WordId {
    type Output = WordId;

    fn bitor(self, rhs: Self) -> Self::Output {
        WordId(self.0 | rhs.0)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WordIdSet(WordId);

impl WordIdSet {
    pub fn new(word_id: WordId) -> Self {
        Self(word_id)
    }

    pub fn unique_count(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn add(&mut self, word_id: WordId) {
        self.0 = self.0 | word_id
    }
}
