use std::cmp::min;

use ratatui::widgets::block::Title;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use regex::Regex;

use crate::state::{App, Panel, Point, VERSION};

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
fn ensure_log_in_viewport(app: &mut App, rect: Rect) {
    if app.matches_should_locate && app.matches_selected.is_some() {
        let match_i = app.matches[app.matches_selected.unwrap()];

        if match_i < rect.height as usize / 2 {
            app.log_offset.y = 0;
        } else if match_i >= app.log_lines.len() - rect.height as usize / 2 {
            app.log_offset.y = app.log_lines.len() - rect.height as usize;
        } else {
            app.log_offset.y = match_i - rect.height as usize / 2;
        }

        app.matches_should_locate = false;
    }
}

fn render_log_text(app: &mut App, rect: Rect) -> Text {
    ensure_log_in_viewport(app, rect);

    let text_lines = cut_text_window(&app.log_lines, &rect, &app.log_offset);

    let mut colored_lines: Vec<Line> = Vec::with_capacity(rect.height as usize);

    for (i, line) in text_lines.iter().enumerate() {
        let highlight = if let Some(match_i) = app.matches_selected {
            app.matches[match_i] == app.log_offset.y + i
        } else {
            false
        };
        colored_lines.push(color_line(&app.re, line, highlight, rect.width));
    }

    Text::from(colored_lines)
}

fn ensure_matches_in_viewport(app: &mut App, rect: Rect) {
    if app.matches_selected.is_none() {
        return;
    }
    let selected = app.matches_selected.unwrap();
    if selected < app.matches_offset.y {
        app.matches_offset.y = selected;
    } else if selected >= app.matches_offset.y + rect.height as usize {
        app.matches_offset.y = selected - rect.height as usize + 1;
    }
}

fn render_matches_text(app: &mut App, rect: Rect) -> Text {
    // TODO: this whole fn probably should be refactored
    ensure_matches_in_viewport(app, rect);

    let matches: Vec<&str> = app.matches.iter().map(|i| &app.log_lines[*i][..]).collect();
    let text_lines = cut_text_window2(matches, &rect, &app.matches_offset);

    let mut colored_lines: Vec<Line> = Vec::with_capacity(rect.height as usize);

    for (i, line) in text_lines.iter().enumerate() {
        let highlight = (app.selected_panel == Panel::Matches) && (app.matches_selected == Some(i));
        colored_lines.push(color_line(&app.re, line, highlight, rect.width));
    }

    Text::from(colored_lines)
}

pub fn render_ui(app: &mut App, frame: &mut Frame) {
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
        Paragraph::new(render_log_text(app, log_block.inner(main_layout[0]))).block(log_block),
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
    if app.re.is_none() {
        search_block = search_block.style(Style::default().fg(Color::Red));
    }
    if app.selected_panel == Panel::Search {
        search_block = search_block.border_style(highlight_style);
    }
    frame.render_widget(
        Paragraph::new(app.search_query.clone()).block(search_block),
        sub_layout[0],
    );

    // matches
    let mut matches_block = Block::new()
        .borders(Borders::TOP)
        .title(Title::from(" Matches ").alignment(Alignment::Center));

    if app.selected_panel == Panel::Matches {
        matches_block = matches_block.border_style(highlight_style);
    }
    frame.render_widget(
        Paragraph::new(render_matches_text(app, matches_block.inner(sub_layout[1])))
            .block(matches_block),
        sub_layout[1],
    );
}
