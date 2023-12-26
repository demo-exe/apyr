use std::cmp::min;

use ratatui::widgets::block::Title;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use regex::Regex;

use crate::types::{Panel, Point, SharedState, UIState, VERSION};

// TODO: refactor into 1 function somehow ? (unsure about lifetimes w/ generics)
fn cut_text_window<'a>(source: &'a Vec<String>, rect: &Rect, offset: &Point) -> Vec<&'a str> {
    let mut text_lines: Vec<&str> = Vec::with_capacity(rect.height as usize);

    let available_lines = min(rect.height as usize, source.len().saturating_sub(offset.y));

    for line in source.iter().skip(offset.y).take(available_lines) {
        let available_width = min(rect.width as usize, line.len().saturating_sub(offset.x));

        if available_width != 0 {
            text_lines.push(&line[offset.x..offset.x + available_width]);
        } else {
            text_lines.push("");
        }
    }

    text_lines
}

fn cut_text_window2<'a>(source: Vec<&'a str>, rect: &Rect, offset: &Point) -> Vec<&'a str> {
    let mut text_lines: Vec<&str> = Vec::with_capacity(rect.height as usize);

    let available_lines = min(rect.height as usize, source.len().saturating_sub(offset.y));

    for line in source.iter().skip(offset.y).take(available_lines) {
        let available_width = min(rect.width as usize, line.len().saturating_sub(offset.x));

        if available_width != 0 {
            text_lines.push(&line[offset.x..offset.x + available_width]);
        } else {
            text_lines.push("");
        }
    }

    text_lines
}

fn color_line<'a>(re: &Option<Regex>, line: &'a str, highlight: bool, width: u16) -> Line<'a> {
    let style = Style::default().fg(Color::Red);
    let hlcolor = Color::DarkGray;

    let not_colored = |start: usize, end: usize| {
        let text = &line[start..end];
        if highlight {
            Span::styled(text, Style::default().bg(hlcolor))
        } else {
            Span::raw(text)
        }
    };

    let colored = |start: usize, end: usize| {
        let text = &line[start..end];
        if highlight {
            Span::styled(text, style.bg(hlcolor))
        } else {
            Span::styled(text, style)
        }
    };

    let mat;
    if let Some(re) = re {
        // TODO: this should not be called if line is not a match
        mat = re
            .find_iter(line)
            .map(|m| (m.start(), m.end()))
            .collect::<Vec<_>>();
    } else {
        if highlight {
            return Line::styled(line, Style::default().bg(hlcolor));
        }
        return Line::raw(line);
    }

    let mut result: Vec<Span> = Vec::new();
    let mut last = 0;

    for m in mat {
        if m.0 > last {
            result.push(not_colored(last, m.0));
        }
        result.push(colored(m.0, m.1));
        last = m.1;
    }
    if last < line.len() {
        result.push(not_colored(last, line.len()));
    }

    if highlight {
        let filler: String = " ".repeat((width.saturating_sub(line.len() as u16)).into());
        result.push(Span::styled(filler, Style::default().bg(hlcolor)));
    }

    Line::from(result)
}
fn ensure_log_in_viewport(app: &SharedState, ui: &mut UIState, rect: Rect) {
    let matches = app.matches.lock().unwrap();
    let log_lines = app.logbuf.tmp_read();
    if ui.matches_should_locate && ui.matches_selected.is_some() {
        let match_i = matches[ui.matches_selected.unwrap()].lineno;

        if match_i < ((rect.height as usize) / 2) {
            ui.log_offset.y = 0;
        } else if match_i >= log_lines.len() - rect.height as usize / 2 {
            // TODO: sus -1 here
            ui.log_offset.y = log_lines.len() - (rect.height as usize - 1);
        } else {
            ui.log_offset.y = match_i - rect.height as usize / 2;
        }

        ui.matches_should_locate = false;
    }
    if ui.following {
        // TODO: probably not a place for it
        ui.matches_selected = None;
        ui.log_offset.y = log_lines.len().saturating_sub(rect.height as usize);
    }
}

fn render_log_text<'a>(
    app: &SharedState,
    ui: &mut UIState,
    log_lines: &'a Vec<String>,
    rect: Rect,
) -> Text<'a> {
    ensure_log_in_viewport(app, ui, rect);

    let matches = app.matches.lock().unwrap();
    // let log_lines = &app.log_lines.read().unwrap();
    let re = app.search.read().unwrap().re.clone();

    let text_lines = cut_text_window(log_lines, &rect, &ui.log_offset);

    let mut colored_lines: Vec<Line> = Vec::with_capacity(rect.height as usize);

    for (i, line) in text_lines.iter().enumerate() {
        let highlight = if let Some(match_i) = ui.matches_selected {
            matches[match_i].lineno == ui.log_offset.y + i
        } else {
            false
        };
        colored_lines.push(color_line(&re, line, highlight, rect.width));
    }

    Text::from(colored_lines)
}

fn ensure_matches_in_viewport(app: &SharedState, ui: &mut UIState, rect: Rect) {
    if ui.matches_selected.is_none() {
        if ui.following {
            // TODO: probably not a place for it
            let matches = app.matches.lock().unwrap();
            ui.matches_offset.y = matches.len().saturating_sub(rect.height as usize);
        }
        return;
    }
    let selected = ui.matches_selected.unwrap();
    {
        let matches = app.matches.lock().unwrap();

        if selected < ui.matches_offset.y {
            ui.matches_offset.y = selected;
            // TODO: double check this
        } else if selected >= ui.matches_offset.y + rect.height as usize {
            ui.matches_offset.y = selected - (rect.height as usize) / 2 + 1;
        }

        if selected < ((rect.height as usize) / 2) {
            ui.matches_offset.y = 0;
        } else if selected >= matches.len() - rect.height as usize / 2 {
            // TODO: sus -1 here, should be rewritten
            ui.matches_offset.y = matches.len().saturating_sub(rect.height as usize - 1);
        } else {
            ui.matches_offset.y = selected - rect.height as usize / 2;
        }
    }
}

fn render_matches_text<'a>(
    app: &SharedState,
    ui: &mut UIState,
    log_lines: &'a [String],
    rect: Rect,
) -> Text<'a> {
    // TODO: this whole fn probably should be refactored
    ensure_matches_in_viewport(app, ui, rect);

    // let log_lines = &app.log_lines.read().unwrap();
    let matches: Vec<_> = app
        .matches
        .lock()
        .unwrap()
        .iter()
        .map(|i| &log_lines[(*i).lineno][..])
        .collect();
    let text_lines = cut_text_window2(matches, &rect, &ui.matches_offset);

    let mut colored_lines: Vec<Line> = Vec::with_capacity(rect.height as usize);

    let re = app.search.read().unwrap().re.clone();
    for (i, line) in text_lines.iter().enumerate() {
        let highlight = (ui.selected_panel == Panel::Matches)
            && (ui.matches_selected == Some(i + ui.matches_offset.y));
        colored_lines.push(color_line(&re, line, highlight, rect.width));
    }

    Text::from(colored_lines)
}

pub fn render_ui(app: &SharedState, ui: &mut UIState, frame: &mut Frame) {
    // default colors TODO: extract to some config
    let highlight_style = Style::default().bold().fg(Color::White);

    // top
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Min(5)])
        .split(frame.size());

    // log window
    let log_block = Block::default()
        .borders(Borders::TOP)
        .title(Title::from(" Log {stdin} ").alignment(Alignment::Center))
        .title(Title::from(format!(" Apyr v{VERSION}")).alignment(Alignment::Right));
    frame.render_widget(
        Paragraph::new(render_log_text(
            app,
            ui,
            &app.logbuf.tmp_read(),
            log_block.inner(main_layout[0]),
        ))
        .block(log_block),
        main_layout[0],
    );

    // bottom cluster = search + matches
    let sub_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(main_layout[1]);
    let mut search_block = Block::default()
        .borders(Borders::TOP)
        .title(Title::from(" Search  ").alignment(Alignment::Center));
    if app.search.read().unwrap().re.is_none() {
        search_block = search_block.style(Style::default().fg(Color::Red));
    }
    if ui.selected_panel == Panel::Search {
        search_block = search_block.border_style(highlight_style);
    }
    frame.render_widget(
        Paragraph::new(ui.search_query.clone()).block(search_block),
        sub_layout[0],
    );

    // matches
    let mut matches_block = Block::new()
        .borders(Borders::TOP)
        .title(Title::from(" Matches ").alignment(Alignment::Center));

    if ui.selected_panel == Panel::Matches {
        matches_block = matches_block.border_style(highlight_style);
    }
    frame.render_widget(
        Paragraph::new(render_matches_text(
            app,
            ui,
            &app.logbuf.tmp_read(),
            matches_block.inner(sub_layout[1]),
        ))
        .block(matches_block),
        sub_layout[1],
    );
}
