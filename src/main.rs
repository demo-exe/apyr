use anyhow::Result;

#[cfg(debug_assertions)]
use better_panic::Settings;

use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event::Event::Key, execute};
use ratatui::prelude::{CrosstermBackend, Terminal};
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::{Signals, SignalsInfo};
use state::App;

mod reader;
mod state;
mod ui;

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
    std::panic::set_hook(Box::new(|panic_info| {
        shutdown().unwrap();
        Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(panic_info);
    }));
}

#[cfg(not(debug_assertions))]
pub fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        shutdown().unwrap();
        original_hook(panic_info);
    }));
}

// App update function
fn process_event(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.modifiers == event::KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                } else {
                    state::process_key_event(key, app);
                }
            }
        }
    }
    Ok(())
}

fn run(mut signals: SignalsInfo) -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App::default();

    loop {
        t.draw(|f| {
            ui::render_ui(&app, f);
        })?;

        process_event(&mut app)?;

        if app.should_quit || signals.pending().next().is_some() {
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
