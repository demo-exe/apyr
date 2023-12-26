use std::sync::{atomic::Ordering, Arc};

use crossbeam::channel;

use crate::types::{Match, SearchCriteria, SharedState};

pub fn worker_thread(
    app_handle: Arc<SharedState>,
    channel: channel::Receiver<(usize, usize)>,
) -> ! {
    let mut version = 0;
    let mut regex = SearchCriteria { re: None };
    loop {
        let range = channel.recv().unwrap();

        let current = app_handle.search_version.load(Ordering::Relaxed);
        if current != version {
            // update version and regex
            version = current;
            regex = app_handle.search.read().unwrap().clone();
        }

        let mut matches = Vec::new();

        if let Some(re) = &regex.re {
            let log_lines = app_handle.logbuf.tmp_read();
            for i in range.0..range.1 {
                if re.is_match(&log_lines[i]) {
                    matches.push(Match {
                        lineno: i,
                        // TODO: still needed here ?
                        version,
                    });
                }
            }
        }

        if !matches.is_empty() {
            app_handle.matches_channel_send.send(matches).unwrap();
        }
    }
}
