use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Mutex, RwLock,
};

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

#[derive(Clone)]
pub struct Match {
    pub lineno: usize,
    pub version: usize,
}

// State shared between threads
pub struct SharedState {
    pub should_quit: AtomicBool,

    pub logbuf: LogBuf,

    pub search: RwLock<SearchCriteria>,
    pub search_version: AtomicUsize,

    // vec of line numbers
    pub matches: Mutex<Vec<Match>>,

    pub regex_channel: channel::Sender<(usize, usize)>,
    pub matches_channel_send: channel::Sender<Vec<Match>>,
    pub matches_channel_recv: channel::Receiver<Vec<Match>>,
}

impl SharedState {
    pub fn new(
        regex_channel: channel::Sender<(usize, usize)>,
        matches_channel_send: channel::Sender<Vec<Match>>,
        matches_channel_recv: channel::Receiver<Vec<Match>>,
    ) -> Self {
        SharedState {
            should_quit: AtomicBool::new(false),

            logbuf: LogBuf::new(),

            search: RwLock::new(SearchCriteria { re: None }),
            search_version: AtomicUsize::new(0),

            matches: Mutex::new(Vec::new()),

            regex_channel,
            matches_channel_send,
            matches_channel_recv,
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
}
