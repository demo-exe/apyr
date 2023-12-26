use std::sync::{atomic::Ordering, Arc};

use crate::types::SharedState;

// This threads inserts matches coming from worker threads into the matches vector
pub fn sorter_thread(app: Arc<SharedState>) {
    let mut current_version = 0;
    loop {
        // matches will always be sorted by lineno
        let matches = app.matches_channel_recv.recv().unwrap();

        // should not happen
        if matches.is_empty() {
            continue;
        }

        let mut app_matches = app.matches.lock().unwrap();

        // during waiting for the lock, version might have changed
        let actual = app.search_version.load(Ordering::Relaxed);
        if current_version < actual {
            current_version = actual;
        }

        if matches[0].version != current_version {
            continue;
        }

        app_matches.reserve(matches.len());

        // TODO: optimize this naive implementation
        for m in matches.iter() {
            let pos = app_matches.binary_search_by_key(&m.lineno, |m| m.lineno);
            if let Err(pos) = pos {
                app_matches.insert(pos, m.clone());
            }
        }
    }
}
