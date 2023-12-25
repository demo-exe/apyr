use std::sync::Arc;

use crossbeam::channel;

use crate::types::{Match, SharedState};

pub fn worker_thread(app_handle: Arc<SharedState>, channel: channel::Receiver<(usize, usize)>) {
    loop {
        let range = channel.recv().unwrap();

        let re;
        {
            re = app_handle.search.read().unwrap().clone();
        }

        let mut matches = Vec::new();

        if let Some(ire) = re.re {
            let log_lines = app_handle.logbuf.tmp_read();
            for i in range.0..range.1 {
                if ire.is_match(&log_lines[i]) {
                    matches.push(Match {
                        lineno: i,
                        version: re.version,
                    });
                }
            }
        }

        if !matches.is_empty() {
            app_handle.matches_channel_send.send(matches).unwrap();
        }
    }
}
