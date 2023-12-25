use std::sync::Arc;

use crate::types::SharedState;

// This threads inserts matches coming from worker threads into the matches vector
pub fn sorter_thread(app: Arc<SharedState>) {
    loop {
        let mut matches = app.matches_channel_recv.recv().unwrap();

        let mut app_matches = app.matches.lock().unwrap();
        app_matches.append(&mut matches);
    }
}
