use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use anyhow::Result;

#[cfg(debug_assertions)]
use better_panic::Settings;

use crossbeam::channel;
use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event::Event::Key, execute};
use ratatui::prelude::{CrosstermBackend, Terminal};
use reader::reader_thread;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::{Signals, SignalsInfo};
use sorter::sorter_thread;
use types::{Match, SharedState, UIState};

mod control;
mod logbuf;
mod reader;
mod sorter;
mod types;
mod ui;
mod worker;

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[cfg(debug_assertions)]
pub fn initialize_panic_handler() {
    use std::process::exit;

    std::panic::set_hook(Box::new(|panic_info| {
        let _ = shutdown();
        Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(panic_info);
        exit(1);
    }));
}

#[cfg(not(debug_assertions))]
pub fn initialize_panic_handler() {
    use std::process::exit;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = shutdown();
        original_hook(panic_info);
        exit(1);
    }));
}

// App update function
fn process_event(app: &Arc<SharedState>, ui: &mut UIState) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.modifiers == event::KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    app.should_quit.store(true, Ordering::Relaxed);
                } else {
                    control::process_key_event(key, app, ui);
                }
            }
        }
    }
    Ok(())
}

fn run(mut signals: SignalsInfo) -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut uistate = UIState::default();

    let mut regex_threads = Vec::new();

    let (re_send, re_recv) = channel::unbounded::<(usize, usize)>();
    let (match_send, match_recv) = channel::unbounded::<Vec<Match>>();

    let app = Arc::new(SharedState::new(re_send, match_send, match_recv));
    let app_handle = app.clone();

    // reader can be permamently blocked by stdin().read_line() so we don't join it
    thread::Builder::new()
        .name("reader".to_string())
        .spawn(move || reader_thread(app_handle))
        .unwrap();

    let app_handle = app.clone();
    thread::Builder::new()
        .name("sorter".to_string())
        .spawn(move || sorter_thread(app_handle))
        .unwrap();

    for i in 0..1 {
        let app_handle = app.clone();
        let receiver_handle = re_recv.clone();
        let thread = thread::Builder::new()
            .name(format!("worker-{}", i))
            .spawn(move || worker::worker_thread(app_handle, receiver_handle));
        regex_threads.push(thread);
    }

    loop {
        t.draw(|f| {
            ui::render_ui(&app, &mut uistate, f);
        })?;

        process_event(&app, &mut uistate)?;

        if app.should_quit.load(Ordering::Relaxed) || signals.pending().next().is_some() {
            app.should_quit.store(true, Ordering::Relaxed);
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let signals = Signals::new([SIGTERM, SIGHUP, SIGINT, SIGQUIT])?;

    initialize_panic_handler();

    startup()?;

    let result = run(signals);

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}
