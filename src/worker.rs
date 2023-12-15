use std::sync::Arc;

use crossbeam::channel;

use crate::state::App;

pub fn worker_thread(app_handle: Arc<App>, channel: channel::Receiver<(usize, usize)>) {
    loop {
        let range = channel.recv().unwrap();

        let re;
        {
            re = app_handle.re.read().unwrap().clone();
        }

        let mut matches = Vec::new();

        if let Some(re) = re {
            let log_lines = app_handle.log_lines.read().unwrap();
            for i in range.0..range.1 {
                if re.is_match(&log_lines[i]) {
                    matches.push(i);
                }
            }
        }

        if matches.len() != 0 {
            let mut app_matches = app_handle.matches.lock().unwrap();
            app_matches.append(&mut matches);
        }
    }
}
