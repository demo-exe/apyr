use anyhow::Result;
use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event::Event::Key, execute};
use ratatui::prelude::{CrosstermBackend, Terminal};
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

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // application state
    let mut app = App::default();

    loop {
        // application render
        t.draw(|f| {
            ui::render_ui(&app, f);
        })?;

        // application update
        process_event(&mut app)?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // setup terminal
    startup()?;

    let result = run();

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}
