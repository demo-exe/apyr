use std::{
    io,
    sync::{atomic::Ordering, Arc},
};

use crossterm::tty::IsTty;

use crate::types::SharedState;

// pub fn read_file() -> Vec<String> {
//     let mut file = File::open("src/main.rs").unwrap();
//     let mut contents = String::new();
//     file.read_to_string(&mut contents).unwrap();
//
//     let mut lines = Vec::new();
//     for line in contents.lines() {
//         lines.push(line.to_string());
//     }
//
//     lines
// }

#[inline(always)]
pub fn push_line(app: &Arc<SharedState>, line: String, line_count: usize) {
    {
        let mut lock = app.log_lines.write().unwrap();
        lock.push(line);
    }
    app.regex_channel
        .send((line_count, line_count + 1))
        .unwrap();
}

pub fn reader_thread(app: Arc<SharedState>) {
    let mut line_count: usize = 0;
    let stdin = io::stdin();
    if stdin.is_tty() {
        // no pipe, no stdin
        push_line(&app, String::from(" ** EOF REACHED ** "), line_count);
        return;
    }

    loop {
        if app.should_quit.load(Ordering::Relaxed) {
            break;
        }
        let mut buffer = String::new();

        let size = stdin.read_line(&mut buffer);

        match size {
            Ok(0) => {
                push_line(&app, String::from(" ** EOF REACHED ** "), line_count);
                break;
            }
            Ok(_) => {
                push_line(&app, buffer, line_count);
                line_count += 1;
            }
            Err(_) => {
                todo!("Handle me!");
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
