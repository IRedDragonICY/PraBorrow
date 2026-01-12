use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};
use std::{error::Error, io, path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Clone)]
enum Mode {
    /// Online mode connecting to a running node via gRPC
    Online {
        #[arg(short, long, default_value = "http://127.0.0.1:50051")]
        address: String,
    },
    /// Offline mode inspecting a local database
    Offline {
        #[arg(short, long)]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(cli.mode);
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

struct App {
    mode: Mode,
    tab_index: usize,
    should_quit: bool,
    logs: Vec<String>,
    paused: bool,
    filter_input: String,
    is_typing: bool,
}

impl App {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            tab_index: 0,
            should_quit: false,
            logs: vec![
                "System initialized".to_string(),
                "Ready to inspect".to_string(),
            ],
            paused: false,
            filter_input: String::new(),
            is_typing: false,
        }
    }
}

async fn run_app<B: ratatui::backend::Backend<Error = io::Error>>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.is_typing {
                    match key.code {
                        KeyCode::Enter => app.is_typing = false,
                        KeyCode::Esc => {
                            app.is_typing = false;
                            app.filter_input.clear();
                        }
                        KeyCode::Backspace => {
                            app.filter_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.filter_input.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Tab => {
                            app.tab_index = (app.tab_index + 1) % 3;
                        }
                        KeyCode::Char('p') => {
                            app.paused = !app.paused;
                        }
                        KeyCode::Char('/') => {
                            app.is_typing = true;
                        }
                        KeyCode::Esc => {
                            app.filter_input.clear();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Header
    let mode_str = match &app.mode {
        Mode::Online { address } => format!("Online Mode: {}", address),
        Mode::Offline { path } => format!("Offline Mode: {:?}", path),
    };

    let status = if app.paused { "PAUSED" } else { "RUNNING" };
    let header_text = format!("PraBorrow Dashboard - {} [{}]", mode_str, status);

    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Main Content (Tabs)
    let titles: Vec<Line> = ["Overview", "Log Explorer", "Deadlocks"]
        .iter()
        .cloned()
        .map(Line::from)
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.tab_index)
        .block(Block::default().title("View").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // Split main area into tabs and content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    frame.render_widget(tabs, main_chunks[0]);

    match app.tab_index {
        0 => render_overview(frame, main_chunks[1], app),
        1 => render_log_explorer(frame, main_chunks[1], app),
        2 => render_deadlocks(frame, main_chunks[1], app),
        _ => {}
    }

    // Footer
    // Footer
    let footer_text = if app.is_typing {
        format!("Filter: {}_", app.filter_input)
    } else if !app.filter_input.is_empty() {
        format!("Filter: {} (Press '/' to edit, 'Esc' to clear) | 'p' Pause | 'q' Quit", app.filter_input)
    } else {
        "Press '/' to filter, 'p' to pause, 'q' to quit, 'Tab' to switch views".to_string()
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn render_overview(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, _app: &App) {
    let text = vec![
        Line::from(Span::raw("System Status: Operational")),
        Line::from(Span::raw("Nodes: 3/3 Online")),
        Line::from(Span::raw("Consensus: Stable")),
    ];
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title("Cluster Overview")
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, area);
}

fn render_log_explorer(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    let items: Vec<ListItem> = app
        .logs
        .iter()
        .filter(|log| {
            if app.filter_input.is_empty() {
                true
            } else {
                log.to_lowercase().contains(&app.filter_input.to_lowercase())
            }
        })
        .map(|log| ListItem::new(Line::from(Span::raw(log))))
        .collect();

    let list = List::new(items).block(Block::default().title("Raft Logs").borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn render_deadlocks(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, _app: &App) {
    let paragraph = Paragraph::new("No deadlocks detected in Sovereign resource graph.")
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .title("Deadlock Detector")
                .borders(Borders::ALL),
        );
    frame.render_widget(paragraph, area);
}
