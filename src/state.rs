use crossterm::event::{KeyCode, KeyEvent};
use regex::Regex;

use crate::reader;

pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Panel {
    Log,
    Search,
    Matches,
}

// App state
pub struct App {
    pub should_quit: bool,
    pub lines: Vec<String>,
    pub cursor: Point,
    pub re: Regex,
    pub selected_panel: Panel,
    pub last_panel: Panel,
    pub search_query: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            should_quit: false,
            lines: reader::read_file(),
            cursor: Point { x: 0, y: 0 },
            re: Regex::new(r"App").unwrap(),
            selected_panel: Panel::Search,
            last_panel: Panel::Log,
            search_query: String::new(),
        }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn process_key_event(key: KeyEvent, app: &mut App) {
    if app.selected_panel == Panel::Search {
        // TODO: vi mode ? how to best
        if let KeyCode::Char(c) = key.code {
            app.search_query.push(c);
        } else if key.code == KeyCode::Backspace {
            app.search_query.pop();
        } else if key.code == KeyCode::Esc {
            app.selected_panel = app.last_panel;
        }
    } else {
        match key.code {
            KeyCode::Char('j') => app.cursor.y = app.cursor.y.saturating_add(1),
            KeyCode::Char('k') => app.cursor.y = app.cursor.y.saturating_sub(1),
            KeyCode::Char('u') => app.cursor.y = app.cursor.y.saturating_sub(5),
            KeyCode::Char('d') => app.cursor.y = app.cursor.y.saturating_add(5),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Tab => {
                app.selected_panel = match app.selected_panel {
                    Panel::Log => Panel::Matches,
                    Panel::Matches => Panel::Log,
                    _ => app.selected_panel,
                }
            }
            KeyCode::Char('i') => {
                app.last_panel = app.selected_panel;
                app.selected_panel = Panel::Search;
            }
            _ => {}
        }
    }
}