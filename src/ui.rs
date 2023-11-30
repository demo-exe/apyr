use ratatui::widgets::block::Title;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};

use crate::state::{App, Panel, VERSION};

fn cut_text_window(app: &App, rect: Rect) -> Vec<String> {
    let mut text_lines = Vec::new();

    if app.cursor.y >= app.lines.len() as u32 {
        return text_lines;
    }

    let mut fitting = app.lines.len() - app.cursor.y as usize;
    if fitting > rect.height as usize {
        fitting = rect.height as usize;
    }

    for i in 0..fitting {
        let line = &app.lines[(app.cursor.y + i as u32) as usize];
        text_lines.push(line.clone());
    }
    text_lines
}

fn color_lines<'a>(app: &App, lines: Vec<String>) -> Text<'a> {
    let mut colored_lines = Vec::new();
    let style = Style::default().fg(Color::Red);

    for line in lines {
        if app.re.is_none() {
            colored_lines.push(Line::raw(line));
            continue;
        }
        let mat = app
            .re
            .as_ref()
            .unwrap()
            .find_iter(&line)
            .collect::<Vec<_>>();

        if mat.len() > 0 {
            let mut styledline: Vec<Span> = Vec::new();

            let mut printed: usize = 0;
            for mat in mat {
                if mat.start() > printed {
                    styledline.push(Span::raw((&line[printed..mat.start()]).to_string()));
                }
                let colored = mat.as_str().to_string();
                styledline.push(Span::styled(colored, style));
                printed = mat.end();
            }
            if printed < line.len() {
                styledline.push(Span::raw((&line[printed..]).to_string()));
            }
            colored_lines.push(Line::from(styledline));
        } else {
            colored_lines.push(Line::raw(line));
        }
    }
    Text::from(colored_lines)
}

pub fn render_ui(app: &App, frame: &mut Frame) {
    let highlight_style = Style::default().bold().fg(Color::White);
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Min(5)])
        .split(frame.size());

    let mut block = Block::default()
        .borders(Borders::TOP)
        .title(Title::from(" Log {stdin} ").alignment(Alignment::Center))
        .title(Title::from(format!(" Apyr v{VERSION}")).alignment(Alignment::Right));

    if app.selected_panel == Panel::Log {
        block = block.border_style(highlight_style);
    }

    frame.render_widget(
        Paragraph::new(color_lines(
            &app,
            cut_text_window(&app, block.inner(main_layout[0])),
        ))
        .block(block),
        main_layout[0],
    );

    let sub_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(main_layout[1]);

    let mut block = Block::default()
        .borders(Borders::TOP)
        .title(Title::from(" Search  ").alignment(Alignment::Center));
    if app.selected_panel == Panel::Search {
        block = block.border_style(highlight_style);
    }
    frame.render_widget(
        Paragraph::new(app.search_query.clone()).block(block),
        sub_layout[0],
    );

    let mut block = Block::new()
        .borders(Borders::TOP)
        .title(Title::from(" Matches ").alignment(Alignment::Center));

    if app.selected_panel == Panel::Matches {
        block = block.border_style(highlight_style);
    }

    frame.render_widget(block, sub_layout[1]);
}
