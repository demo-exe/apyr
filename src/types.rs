use std::sync::{atomic::AtomicBool, Mutex, RwLock};

use crossbeam::channel;
use regex::Regex;

use crate::logbuf::LogBuf;

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

// State shared between threads
pub struct SharedState {
    pub should_quit: AtomicBool,

    pub logbuf: LogBuf,

    pub search: RwLock<SearchCriteria>,

    // vec of line numbers
    pub matches: Mutex<Vec<usize>>,

    pub regex_channel: channel::Sender<(usize, usize)>,
}

impl SharedState {
    pub fn new(regex_channel: channel::Sender<(usize, usize)>) -> Self {
        SharedState {
            should_quit: AtomicBool::new(false),

            logbuf: LogBuf::new(),

            search: RwLock::new(SearchCriteria {
                re: None,
                version: 0,
            }),

            matches: Mutex::new(Vec::new()),

            regex_channel,
        }
    }
}

// Owned by the UI thread and not shared
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

// TODO: maybe use arc_swap crate instead?
// only one thread is writing to this
#[derive(Clone)]
pub struct SearchCriteria {
    pub re: Option<Regex>,
    pub version: usize,
}
