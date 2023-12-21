use std::sync::{atomic::AtomicBool, Mutex, RwLock};

use crossbeam::channel;
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
    pub should_quit: AtomicBool,

    pub log_lines: RwLock<Vec<String>>,

    pub re: RwLock<Option<Regex>>,
    // vec of line numbers
    pub matches: Mutex<Vec<usize>>,

    pub regex_channel: channel::Sender<(usize, usize)>,
}

pub struct UIState {
    pub log_offset: Point,
    pub log_max_width: usize,

    pub selected_panel: Panel,

    pub matches_selected: Option<usize>,
    pub matches_should_locate: bool,
    pub matches_offset: Point,

    pub search_query: String,

    pub following: bool,
}

impl Default for UIState {
    fn default() -> Self {
        UIState {
            log_offset: Point::default(),
            log_max_width: 0,

            selected_panel: Panel::Search,

            matches_selected: None,
            matches_should_locate: false,
            matches_offset: Point::default(),

            search_query: String::new(),

            following: true,
        }
    }
}

impl App {
    pub fn new(regex_channel: channel::Sender<(usize, usize)>) -> Self {
        App {
            should_quit: AtomicBool::new(false),

            log_lines: RwLock::new(Vec::with_capacity(1024)),

            re: RwLock::new(None),

            matches: Mutex::new(Vec::new()),

            regex_channel,
        }
    }
}

fn recompile_regex(app: &App, ui: &mut UIState) {
    // TODO: this will probably not work in some race conditions (channel not empty)
    let mut matches = app.matches.lock().unwrap();
    matches.clear();
    ui.matches_selected = None;
    ui.matches_offset = Point::default();

    if ui.search_query.len() < 3 {
        *app.re.write().unwrap() = None;
        return;
    }

    let re = Regex::new(&ui.search_query);
    if let Ok(re) = re {
        *app.re.write().unwrap() = Some(re);
    }

    {
        let log_lines = app.log_lines.read().unwrap();

        // chunks of 10
        (0..log_lines.len()).step_by(10).for_each(|i| {
            let last = std::cmp::min(i + 10, log_lines.len());
            app.regex_channel.send((i, last)).unwrap();
        });
    }
}

fn add_matches_scroll(app: &App, ui: &mut UIState, value: isize) {
    let matches = app.matches.lock().unwrap();
    if matches.is_empty() {
        return;
    }
    ui.following = false;
    ui.matches_should_locate = true;
    if let Some(selected) = ui.matches_selected {
        ui.matches_selected = Some(selected.saturating_add_signed(value));
        if ui.matches_selected.unwrap() >= matches.len() {
            ui.matches_selected = Some(matches.len() - 1);
        }
    } else {
        ui.matches_selected = Some(ui.matches_offset.y);
    }
}

fn add_log_scroll(_app: &App, ui: &mut UIState, value: isize) {
    ui.log_offset.y = ui.log_offset.y.saturating_add_signed(value);
}

fn add_horizontal_scroll(_app: &App, ui: &mut UIState, value: isize) {
    ui.log_offset.x = ui.log_offset.x.saturating_add_signed(value);
    ui.matches_offset.x = ui.log_offset.x;
}

pub fn process_key_event(key: KeyEvent, app: &App, ui: &mut UIState) {
    // common
    #[allow(clippy::single_match)]
    match key.code {
        KeyCode::Tab => {
            match ui.selected_panel {
                Panel::Search => ui.selected_panel = Panel::Matches,
                Panel::Matches => {
                    ui.selected_panel = Panel::Search;
                }
            };
        }
        _ => {}
    }
    match ui.selected_panel {
        Panel::Search => {
            // TODO: vi mode ? how to best
            if let KeyCode::Char(c) = key.code {
                ui.search_query.push(c);
                recompile_regex(app, ui);
            } else if key.code == KeyCode::Backspace {
                ui.search_query.pop();
                recompile_regex(app, ui);
            } else if key.code == KeyCode::Esc {
                ui.selected_panel = Panel::Matches;
            }
        }
        Panel::Matches => match key.code {
            KeyCode::Char('j') => add_matches_scroll(app, ui, 1),
            KeyCode::Char('k') => add_matches_scroll(app, ui, -1),
            KeyCode::Char('d') => add_log_scroll(app, ui, 5),
            KeyCode::Char('u') => add_log_scroll(app, ui, -5),
            KeyCode::Char('l') => add_horizontal_scroll(app, ui, 3),
            KeyCode::Char('h') => add_horizontal_scroll(app, ui, -3),
            KeyCode::Char('q') => app
                .should_quit
                .store(true, std::sync::atomic::Ordering::Relaxed),
            KeyCode::Char('c') => {
                ui.search_query.clear();
                ui.selected_panel = Panel::Search;
            }
            KeyCode::Char('i') => {
                ui.selected_panel = Panel::Search;
            }
            KeyCode::Char('f') => {
                ui.following = true;
            }
            _ => {}
        },
    }
}
