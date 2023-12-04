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
    pub re: Option<Regex>,
    pub selected_panel: Panel,
    pub last_panel: Panel,
    pub search_query: String,
    pub matches: Vec<usize>,

    pub matches_selected: Option<usize>,
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
            last_panel: Panel::Log,
            search_query: String::new(),
            matches: Vec::new(),

            matches_selected: None,
            matches_offset: Point::default(),
        }
    }
}

fn recompile_regex(app: &mut App) {
    app.matches.clear();
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
            if re.find(line).is_some() {
                app.matches.push(i);
            }
        }
    }
}

fn add_matches_scroll(app: &mut App, is_add: bool, value: usize) {
    if app.matches_selected.is_none() {
        app.matches_selected = Some(app.matches_offset.y);
    }
    let selected = app.matches_selected.unwrap();
    if is_add {
        app.matches_selected = Some(selected.saturating_add(value));
    } else {
        app.matches_selected = Some(selected.saturating_sub(value));
    }
}

pub fn process_key_event(key: KeyEvent, app: &mut App) {
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
                app.selected_panel = app.last_panel;
            }
        }
        Panel::Matches => match key.code {
            KeyCode::Char('j') => add_matches_scroll(app, true, 1),
            KeyCode::Char('k') => add_matches_scroll(app, false, 1),
            KeyCode::Char('u') => add_matches_scroll(app, false, 5),
            KeyCode::Char('d') => add_matches_scroll(app, true, 5),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('c') => {
                app.search_query.clear();
                app.selected_panel = Panel::Search;
            }
            KeyCode::Tab => {
                app.selected_panel = match app.selected_panel {
                    Panel::Log => Panel::Matches,
                    Panel::Matches => Panel::Log,
                    _ => app.selected_panel,
                };
            }
            KeyCode::Char('i') => {
                app.last_panel = app.selected_panel;
                app.selected_panel = Panel::Search;
            }
            _ => {}
        },
        Panel::Log => match key.code {
            KeyCode::Char('j') => app.log_offset.y = app.log_offset.y.saturating_add(1),
            KeyCode::Char('k') => app.log_offset.y = app.log_offset.y.saturating_sub(1),
            KeyCode::Char('u') => app.log_offset.y = app.log_offset.y.saturating_sub(5),
            KeyCode::Char('d') => app.log_offset.y = app.log_offset.y.saturating_add(5),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('c') => {
                app.search_query.clear();
                app.selected_panel = Panel::Search;
            }
            KeyCode::Tab => {
                app.selected_panel = match app.selected_panel {
                    Panel::Log => Panel::Matches,
                    Panel::Matches => Panel::Log,
                    _ => app.selected_panel,
                };
            }
            KeyCode::Char('i') => {
                app.last_panel = app.selected_panel;
                app.selected_panel = Panel::Search;
            }
            _ => {}
        },
    }
}
