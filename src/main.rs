use anyhow::Result;
use crossterm::event;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{
    event::{Event::Key, KeyCode::Char},
    execute,
};
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

// App state
struct App {
    should_quit: bool,
    lines: Vec<String>,
    cursor: Point,
    re: Regex,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

// App update function
fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char('j') => app.cursor.y = app.cursor.y.saturating_add(1),
                    Char('k') => app.cursor.y = app.cursor.y.saturating_sub(1),
                    Char('u') => app.cursor.y = app.cursor.y.saturating_sub(5),
                    Char('d') => app.cursor.y = app.cursor.y.saturating_add(5),
                    Char('q') => app.should_quit = true,
                    _ => {}
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
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.size());
    frame.render_widget(
        Block::new().title(format!("Apyr v{VERSION}")),
        main_layout[0],
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(main_layout[1]);
    let block = Block::default().borders(Borders::ALL).title("Log");
    frame.render_widget(
        Paragraph::new(color_lines(
            &app,
            cut_text_window(&app, block.inner(inner_layout[0])),
        ))
        .block(block),
        inner_layout[0],
    );
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Matches"),
        inner_layout[1],
    );
}
