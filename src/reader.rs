use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::state::App;

pub fn read_file() -> Vec<String> {
    let mut file = File::open("src/main.rs").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut lines = Vec::new();
    for line in contents.lines() {
        lines.push(line.to_string());
    }

    lines
}

pub fn reader_thread(app: Arc<Mutex<App>>) {
    let mut i: usize = 0;
    loop {
        thread::sleep(Duration::new(1, 0));
        {
            let mut app = app.lock().unwrap();
            app.log_lines
                .push(format!("[{:>5}]: Line from reader thread", i));
            i += 1;
            if app.should_quit {
                break;
            }
        }
    }
}
