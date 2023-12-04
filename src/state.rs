use crossterm::event::{KeyCode, KeyEvent};
use regex::Regex;

use crate::reader;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Point {
    pub x: usize,
    pub y: usize,
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

    pub log_lines: Vec<String>,
    pub log_offset: Point,

    pub search_query: String,
    pub re: Option<Regex>,
    // vec of line numbers
    pub matches: Vec<usize>,

    pub selected_panel: Panel,

    pub matches_selected: Option<usize>,
    pub matches_selected_last: Option<usize>,
    pub matches_offset: Point,
}

impl Default for App {
    fn default() -> Self {
        App {
            should_quit: false,
            log_lines: reader::read_file(),
            log_offset: Point::default(),
            re: None,
            selected_panel: Panel::Search,
            search_query: String::new(),
            matches: Vec::new(),

            matches_selected: None,
            matches_selected_last: None,
            matches_offset: Point::default(),
        }
    }
}

fn recompile_regex(app: &mut App) {
    app.matches.clear();
    app.matches_selected = None;
    app.matches_offset = Point::default();

    if app.search_query.len() <= 3 {
        app.re = None;
        return;
    }

    app.re = Regex::new(&app.search_query).ok();

    // find matches
    if app.search_query.is_empty() {
        return;
    }
    if let Some(re) = &app.re {
        for (i, line) in app.log_lines.iter().enumerate() {
            if re.is_match(line) {
                app.matches.push(i);
            }
        }
    }
}

fn add_matches_scroll(app: &mut App, value: isize) {
    if app.matches.len() == 0 {
        return;
    }
    if let Some(selected) = app.matches_selected {
        app.matches_selected = Some(selected.saturating_add_signed(value));
        if app.matches_selected.unwrap() >= app.matches.len() {
            app.matches_selected = Some(app.matches.len() - 1);
            return;
        }
    } else {
        app.matches_selected = Some(app.matches_offset.y);
    }
}

pub fn process_key_event(key: KeyEvent, app: &mut App) {
    // common
    match key.code {
        KeyCode::Tab => {
            match app.selected_panel {
                Panel::Search => app.selected_panel = Panel::Matches,
                Panel::Matches => {
                    app.selected_panel = Panel::Search;
                }
                _ => {
                    panic!("unexpected panel");
                }
            };
        }
        _ => {}
    }
    match app.selected_panel {
        Panel::Search => {
            // TODO: vi mode ? how to best
            if let KeyCode::Char(c) = key.code {
                app.search_query.push(c);
                recompile_regex(app);
            } else if key.code == KeyCode::Backspace {
                app.search_query.pop();
                recompile_regex(app);
            } else if key.code == KeyCode::Esc {
                app.selected_panel = Panel::Matches;
            }
        }
        Panel::Matches => match key.code {
            KeyCode::Char('j') => add_matches_scroll(app, 1),
            KeyCode::Char('k') => add_matches_scroll(app, -1),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('c') => {
                app.search_query.clear();
                app.selected_panel = Panel::Search;
            }
            KeyCode::Char('i') => {
                app.selected_panel = Panel::Search;
            }
            _ => {}
        },
        _ => {
            panic!("unexpected panel");
        }
    }
}
