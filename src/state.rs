use crossterm::event::{KeyCode, KeyEvent};
use regex::Regex;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Panel {
    Search,
    Matches,
}

// App state
pub struct App {
    pub should_quit: bool,

    pub log_lines: Vec<String>,
    pub log_offset: Point,
    pub log_max_width: usize,

    pub search_query: String,
    pub re: Option<Regex>,
    // vec of line numbers
    pub matches: Vec<usize>,

    pub selected_panel: Panel,

    pub matches_selected: Option<usize>,
    pub matches_should_locate: bool,
    pub matches_offset: Point,
}

impl Default for App {
    fn default() -> Self {
        App {
            should_quit: false,

            log_lines: Vec::with_capacity(1024),
            log_offset: Point::default(),
            log_max_width: 0,

            re: None,
            selected_panel: Panel::Search,
            search_query: String::new(),
            matches: Vec::new(),

            matches_selected: None,
            matches_should_locate: false,
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
    if app.matches.is_empty() {
        return;
    }
    app.matches_should_locate = true;
    if let Some(selected) = app.matches_selected {
        app.matches_selected = Some(selected.saturating_add_signed(value));
        if app.matches_selected.unwrap() >= app.matches.len() {
            app.matches_selected = Some(app.matches.len() - 1);
        }
    } else {
        app.matches_selected = Some(app.matches_offset.y);
    }
}

fn add_log_scroll(app: &mut App, value: isize) {
    app.log_offset.y = app.log_offset.y.saturating_add_signed(value);
}

fn add_horizontal_scroll(app: &mut App, value: isize) {
    app.log_offset.x = app.log_offset.x.saturating_add_signed(value);
    app.matches_offset.x = app.log_offset.x;
}

pub fn process_key_event(key: KeyEvent, app: &mut App) {
    // common
    #[allow(clippy::single_match)]
    match key.code {
        KeyCode::Tab => {
            match app.selected_panel {
                Panel::Search => app.selected_panel = Panel::Matches,
                Panel::Matches => {
                    app.selected_panel = Panel::Search;
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
            KeyCode::Char('d') => add_log_scroll(app, 5),
            KeyCode::Char('u') => add_log_scroll(app, -5),
            KeyCode::Char('l') => add_horizontal_scroll(app, 3),
            KeyCode::Char('h') => add_horizontal_scroll(app, -3),
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
    }
}
