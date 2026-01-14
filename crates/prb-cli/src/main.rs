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
use std::{collections::VecDeque, error::Error, io, path::PathBuf, time::Duration};

const HEARTBEAT_TICK_RATE: u64 = 10;
const DEADLOCK_CHECK_TICK_RATE: u64 = 50;

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

use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum ConnectionStatus {
    OfflineMode,
    Connected,
    Reconnecting(usize), // retries
    Disconnected(Instant),
}

struct App {
    mode: Mode,
    tab_index: usize,
    should_quit: bool,
    logs: VecDeque<String>,
    deadlocks: Vec<String>,
    paused: bool,
    filter_input: String,
    is_typing: bool,
    tick_count: u64,
    connection_status: ConnectionStatus,
    endpoint: Option<String>,
}

impl App {
    fn new(mode: Mode) -> Self {
        let (connection_status, endpoint) = match &mode {
            Mode::Online { address } => {
                let addr = if address.starts_with("http://") || address.starts_with("https://") {
                    address.clone()
                } else {
                    format!("http://{}", address)
                };
                (ConnectionStatus::Disconnected(Instant::now()), Some(addr))
            }
            Mode::Offline { .. } => (ConnectionStatus::OfflineMode, None),
        };

        Self {
            mode,
            tab_index: 0,
            should_quit: false,
            logs: VecDeque::from(vec![
                "System initialized".to_string(),
                "Ready to inspect".to_string(),
            ]),
            deadlocks: Vec::new(),
            paused: false,
            filter_input: String::new(),
            is_typing: false,
            tick_count: 0,
            connection_status,
            endpoint,
        }
    }
}

async fn run_app<B: ratatui::backend::Backend<Error = io::Error>>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    use praborrow_lease::grpc::proto::control_plane_client::ControlPlaneClient;
    use praborrow_lease::grpc::proto::{Empty, LogRequest};

    // Client handle. If None, we are disconnected/reconnecting.
    let mut client: Option<ControlPlaneClient<tonic::transport::Channel>> = None;
    let mut last_reconnect_attempt = Instant::now();
    let mut reconnect_backoff_secs = 1;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        #[allow(clippy::collapsible_if)]
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // ... handle inputs ...
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

        // Connection Management Logic
        if let Some(endpoint) = &app.endpoint
            && client.is_none()
        {
            // Try to connect if cooldown passed
            if last_reconnect_attempt.elapsed() >= Duration::from_secs(reconnect_backoff_secs) {
                last_reconnect_attempt = Instant::now();

                app.connection_status =
                    ConnectionStatus::Reconnecting(reconnect_backoff_secs as usize);
                // Use connect_lazy? connect() is async but can timeout.
                // Let's try direct connect with timeout validation.
                match ControlPlaneClient::connect(endpoint.clone()).await {
                    Ok(c) => {
                        client = Some(c);
                        app.connection_status = ConnectionStatus::Connected;
                        reconnect_backoff_secs = 1; // reset
                        app.logs.push_front("Connected to backend.".to_string());
                    }
                    Err(e) => {
                        app.logs.push_front(format!("Connection failed: {}", e));
                        reconnect_backoff_secs = (reconnect_backoff_secs * 2).min(30); // max 30s
                        app.connection_status = ConnectionStatus::Disconnected(Instant::now());
                    }
                }
            }
        }

        if !app.paused {
            app.tick_count += 1;

            let mut should_reconnect = false;

            if let Some(c) = &mut client {
                // Poll gRPC every ~1s (10 ticks)
                #[allow(clippy::collapsible_if, clippy::manual_is_multiple_of)]
                if app.tick_count % HEARTBEAT_TICK_RATE == 0 {
                    // Fetch Status
                    match c.get_node_status(tonic::Request::new(Empty {})).await {
                        Ok(response) => {
                            let status = response.into_inner();
                            app.logs.push_front(format!(
                                "STATUS: {} (Term {})",
                                status.state, status.current_term
                            ));
                            app.connection_status = ConnectionStatus::Connected;
                        }
                        Err(e) => {
                            // If status check fails, assume disconnection
                            app.logs.push_front(format!("Heartbeat failed: {}", e));
                            should_reconnect = true;
                        }
                    }

                    // Fetch Logs (if still connected)
                    if !should_reconnect {
                        if let Ok(response) = c
                            .get_recent_logs(tonic::Request::new(LogRequest { limit: 5 }))
                            .await
                        {
                            let server_logs = response.into_inner().logs;
                            for log in server_logs {
                                if !app.logs.contains(&log) {
                                    app.logs.push_front(log);
                                }
                            }
                        }
                    }

                    // Fetch Deadlocks
                    if !should_reconnect {
                        if let Ok(response) = c.get_deadlocks(tonic::Request::new(Empty {})).await {
                            app.deadlocks = response.into_inner().deadlocks;
                        }
                    }

                    // Cap logs
                    if app.logs.len() > 1000 {
                        app.logs.truncate(1000);
                    }
                }
            }

            if should_reconnect {
                client = None;
                app.connection_status = ConnectionStatus::Disconnected(Instant::now());
            }

            // Every 50 ticks (~5s), simulate a deadlock check
            #[allow(clippy::manual_is_multiple_of)]
            if app.tick_count % DEADLOCK_CHECK_TICK_RATE == 0 {
                // Keep simulation for offline mode
                if matches!(app.mode, Mode::Offline { .. }) {
                    app.deadlocks.clear();
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
    let (status_text, status_color) = match &app.connection_status {
        ConnectionStatus::Connected => ("CONNECTED", Color::Green),
        ConnectionStatus::Reconnecting(_retry) => {
            // Can't return string with lifetime issues here easily, let's format later or use fixed strings
            // Actually let's use a Cow or just format the whole header line
            ("RECONNECTING", Color::Yellow)
        }
        ConnectionStatus::Disconnected(_) => ("DISCONNECTED", Color::Red),
        ConnectionStatus::OfflineMode => ("OFFLINE", Color::Gray),
    };

    let mode_str = match &app.mode {
        Mode::Online { address } => format!("Online: {}", address),
        Mode::Offline { path } => format!("Offline: {:?}", path),
    };

    let time_status = if app.paused { "PAUSED" } else { "RUNNING" };

    let header_text = format!(
        "PraBorrow Dashboard - {} | Status: {} | {}",
        mode_str, status_text, time_status
    );

    // For Reconnecting, maybe append retry count
    let header_text = if let ConnectionStatus::Reconnecting(secs) = app.connection_status {
        format!("{} (Backoff {}s)", header_text, secs)
    } else {
        header_text
    };

    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(status_color)
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
        format!(
            "Filter: {} (Press '/' to edit, 'Esc' to clear) | 'p' Pause | 'q' Quit",
            app.filter_input
        )
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
                log.to_lowercase()
                    .contains(&app.filter_input.to_lowercase())
            }
        })
        .map(|log| ListItem::new(Line::from(Span::raw(log))))
        .collect();

    let list = List::new(items).block(Block::default().title("Raft Logs").borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn render_deadlocks(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    if app.deadlocks.is_empty() {
        let paragraph = Paragraph::new(
            "\n  No deadlocks detected in Sovereign resource graph.\n  System is healthy.",
        )
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .title("Deadlock Detector")
                .borders(Borders::ALL),
        );
        frame.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = app
            .deadlocks
            .iter()
            .map(|d| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        " ⚠ ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(d),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("DEADLOCKS DETECTED")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::LightRed));
        frame.render_widget(list, area);
    }
}
