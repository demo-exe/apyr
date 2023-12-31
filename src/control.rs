use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent};
use regex::Regex;

use crate::types::{Panel, SharedState, UIState};

fn update_search(app: &SharedState, re: Option<Regex>) {
    let mut search = app.search.write().unwrap();
    search.re = re;
    // TODO: do I need to hold the lock here?
    app.search_version.fetch_add(1, Ordering::Relaxed);
}

fn recompile_regex(app: &SharedState, ui: &mut UIState) {
    // clear all matches, hold lock while updating search_version
    // TODO: is this the best way to do this?
    let mut matches = app.matches.lock().unwrap();
    matches.clear();
    ui.matches_selected = None;
    ui.matches_offset.y = 0;

    if ui.search_query.len() < 3 {
        update_search(app, None);
        return;
    }

    let re = Regex::new(&ui.search_query);
    if let Ok(re) = re {
        update_search(app, Some(re));
    }

    {
        let log_lines = app.logbuf.tmp_read();

        (0..log_lines.len()).step_by(10).for_each(|i| {
            let end = std::cmp::min(i + 10, log_lines.len());
            app.regex_channel.send((i, end)).unwrap();
        });
    }
}

fn add_matches_scroll(app: &SharedState, ui: &mut UIState, value: isize) {
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

fn add_log_scroll(_app: &SharedState, ui: &mut UIState, value: isize) {
    ui.log_offset.y = ui.log_offset.y.saturating_add_signed(value);
}

fn add_horizontal_scroll(_app: &SharedState, ui: &mut UIState, value: isize) {
    ui.log_offset.x = ui.log_offset.x.saturating_add_signed(value);
    ui.matches_offset.x = ui.log_offset.x;
}

pub fn process_key_event(key: KeyEvent, app: &SharedState, ui: &mut UIState) {
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
