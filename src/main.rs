use anyhow::Result;
use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{
    event::{Event::Key, KeyCode::Char},
    execute,
};
use ratatui::widgets::block::Title;
use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Frame,
};
use regex::Regex;

mod reader;

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

struct Point {
    x: u32,
    y: u32,
}

#[derive(PartialEq, Clone, Copy)]
enum Panel {
    Log,
    Search,
    Matches,
}

// App state
struct App {
    should_quit: bool,
    lines: Vec<String>,
    cursor: Point,
    re: Regex,
    selected_panel: Panel,
    last_panel: Panel,
    search_query: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

// App update function
fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.modifiers == event::KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                } else if app.selected_panel == Panel::Search {
                    // TODO: vi mode ? how to best
                    if let Char(c) = key.code {
                        app.search_query.push(c);
                    } else if key.code == KeyCode::Backspace {
                        app.search_query.pop();
                    } else if key.code == KeyCode::Esc {
                        app.selected_panel = app.last_panel;
                    }
                } else {
                    match key.code {
                        Char('j') => app.cursor.y = app.cursor.y.saturating_add(1),
                        Char('k') => app.cursor.y = app.cursor.y.saturating_sub(1),
                        Char('u') => app.cursor.y = app.cursor.y.saturating_sub(5),
                        Char('d') => app.cursor.y = app.cursor.y.saturating_add(5),
                        Char('q') => app.should_quit = true,
                        KeyCode::Tab => {
                            app.selected_panel = match app.selected_panel {
                                Panel::Log => Panel::Matches,
                                Panel::Matches => Panel::Log,
                                _ => app.selected_panel,
                            }
                        }
                        Char('i') => {
                            app.last_panel = app.selected_panel;
                            app.selected_panel = Panel::Search;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // application state
    let mut app = App {
        should_quit: false,
        lines: reader::read_file(),
        cursor: Point { x: 0, y: 0 },
        re: Regex::new(r"App").unwrap(),
        selected_panel: Panel::Search,
        last_panel: Panel::Log,
        search_query: String::new(),
    };

    loop {
        // application render
        t.draw(|f| {
            ui(&app, f);
        })?;

        // application update
        update(&mut app)?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // setup terminal
    startup()?;

    let result = run();

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}

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
        let mat = app.re.find_iter(&line).collect::<Vec<_>>();
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

fn ui(app: &App, frame: &mut Frame) {
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
        // .style(Style::default().fg(Color::Red))
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
