use ratatui::widgets::block::Title;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use regex::Regex;

use crate::state::{App, Panel, Point, VERSION};

fn cut_text_window<'a>(source: &'a Vec<String>, rect: &Rect, offset: &Point) -> Vec<&'a str> {
    let mut text_lines: Vec<&str> = Vec::with_capacity(rect.height as usize);

    if offset.y >= source.len() {
        return text_lines;
    }

    for i in offset.y..(offset.y + rect.height as usize) as usize {
        if i >= source.len() {
            break;
        }
        if offset.x + rect.width as usize >= source[i].len() {
            text_lines.push(&source[i][offset.x..]);
            continue;
        } else if offset.x >= source[i].len() {
            text_lines.push("");
        }
        text_lines.push(&source[i][offset.x..offset.x + rect.width as usize]);
    }

    text_lines
}

fn color_line<'a>(re: &Option<Regex>, line: &'a str, highlight: bool) -> Line<'a> {
    let hlcolor = Color::DarkGray;
    let mat;
    if let Some(re) = re {
        // TODO: this should not be called if line is not a match
        mat = re
            .find_iter(&line)
            .map(|m| (m.start(), m.end()))
            .collect::<Vec<_>>();
    } else {
        if highlight {
            return Line::styled(line, Style::default().bg(hlcolor));
        }
        return Line::raw(line);
    }

    let style = Style::default().fg(Color::Red);

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

    Line::from(result)
}

fn render_log_text(app: &App, rect: Rect) -> Text {
    let text_lines = cut_text_window(&app.log_lines, &rect, &app.log_offset);

    let mut colored_lines: Vec<Line> = Vec::with_capacity(rect.height as usize);

    for line in text_lines {
        colored_lines.push(color_line(&app.re, line, false));
    }

    Text::from(colored_lines)
}

pub fn render_ui(app: &App, frame: &mut Frame) {
    // default colors TODO: extract to some config
    let highlight_style = Style::default().bold().fg(Color::White);

    // top
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Min(5)])
        .split(frame.size());

    // log window
    let mut log_block = Block::default()
        .borders(Borders::TOP)
        .title(Title::from(" Log {stdin} ").alignment(Alignment::Center))
        .title(Title::from(format!(" Apyr v{VERSION}")).alignment(Alignment::Right));
    if app.selected_panel == Panel::Log {
        log_block = log_block.border_style(highlight_style);
    }
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
    // let matches = render_matches(&app, block.inner(sub_layout[1]));
    let matches = Text::from(vec![Line::raw("asd")]);
    frame.render_widget(Paragraph::new(matches).block(matches_block), sub_layout[1]);
}
