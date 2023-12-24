use std::{
    ops::Range,
    sync::{RwLock, RwLockReadGuard},
};

// A log buffer stores the raw log in single String and a list of line ranges.
// Access is limited to one reading thread and one writing thread.
// Should be cache friendly.
// TODO: make it lock free
#[allow(dead_code)]
pub struct LogBuf {
    raw: RwLock<String>,
    lines: RwLock<Vec<Range<usize>>>,

    // temporary copy of the raw log
    // used for code refactoring
    // TODO: remove this
    tmp_raw: RwLock<Vec<String>>,
}

#[allow(dead_code)]
impl LogBuf {
    pub fn new() -> Self {
        LogBuf {
            raw: RwLock::new(String::new()),
            lines: RwLock::new(Vec::new()),
            tmp_raw: RwLock::new(Vec::new()),
        }
    }

    pub fn write(&self, data: String) {
        let mut raw = self.raw.write().unwrap();
        let mut tmp_raw = self.tmp_raw.write().unwrap();
        raw.push_str(&data);
        tmp_raw.push(data);
    }

    pub fn read<'a>(&'a self) -> RwLockReadGuard<'a, String> {
        self.raw.read().unwrap()
    }

    pub fn tmp_read<'a>(&'a self) -> RwLockReadGuard<'a, Vec<String>> {
        self.tmp_raw.read().unwrap()
    }
}
