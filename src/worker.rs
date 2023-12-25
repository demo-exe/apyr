use std::sync::Arc;

use crossbeam::channel;

use crate::types::SharedState;

pub fn worker_thread(app_handle: Arc<SharedState>, channel: channel::Receiver<(usize, usize)>) {
    loop {
        let range = channel.recv().unwrap();

        let re;
        {
            re = app_handle.search.read().unwrap().re.clone();
        }

        let mut matches = Vec::new();

        if let Some(re) = re {
            let log_lines = app_handle.logbuf.tmp_read();
            for i in range.0..range.1 {
                if re.is_match(&log_lines[i]) {
                    matches.push(i);
                }
            }
        }

        if !matches.is_empty() {
            let mut app_matches = app_handle.matches.lock().unwrap();
            app_matches.append(&mut matches);
        }
    }
}
