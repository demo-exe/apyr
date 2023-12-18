use std::{
    fs::File,
    io::{self, Read},
    sync::{atomic::Ordering, Arc},
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

pub fn reader_thread(app: Arc<App>) {
    let mut line_count: usize = 0;

    loop {
        if app.should_quit.load(Ordering::Relaxed) {
            break;
        }
        let mut buffer = String::new();

        let size = io::stdin().read_line(&mut buffer);

        match size {
            Ok(0) => {
                // TODO
                thread::sleep(Duration::new(0, 1000000));
                continue;
            }
            Ok(_) => {
                // panic!("eof {}", buffer);
                let mut lock = app.log_lines.write().unwrap();
                lock.push(buffer);
                drop(lock);

                app.regex_channel
                    .send((line_count, line_count + 1))
                    .unwrap();

                line_count += 1;
            }
            Err(_) => {
                // TODO
                // panic!("EOF");
                break;
            }
        }
    }
}
// pub fn reader_thread(app: Arc<App>) {
//     let mut i: usize = 0;
//     loop {
//         thread::sleep(Duration::new(0, 10000000));
//         {
//             app.log_lines
//                 .write()
//                 .unwrap()
//                 .push(format!("[{:>5}]: Line from reader thread", i));
//
//             app.regex_channel.send((i, i + 1)).unwrap();
//
//             i += 1;
//             if app.should_quit.load(Ordering::Relaxed) {
//                 break;
//             }
//         }
//     }
// }
