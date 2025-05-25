use std::{error::Error, io, time::{Duration, Instant}};
use crossterm::{event::{self, Event as CEvent, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, Row, Table, TableState, Paragraph}, layout::{Constraint, Layout, Direction}, style::{Style, Modifier}};
use sysinfo::{ProcessExt, System, SystemExt, NetworksExt};
use serde::Deserialize;
use std::fs;
use sysinfo::NetworkExt;

#[derive(Deserialize)]
struct Config {
    refresh_rate: u64,
    default_sort: String,
}

enum SortBy {
    Cpu,
    Mem,
    Name,
}

struct App {
    sys: System,
    last_updated: Instant,
    refresh_rate: Duration,
    search_query: String,
    searching: bool,
    sort_by: SortBy,
    descending: bool,
    table_state: TableState,
}

// Owned copy of process info to avoid borrow conflicts
struct ProcInfo {
    pid: sysinfo::Pid,
    name: String,
    cpu: f32,
    mem: u64,
}

impl App {
    fn new(config: Config) -> Self {
        let sort_by = match config.default_sort.as_str() {
            "mem" => SortBy::Mem,
            "name" => SortBy::Name,
            _ => SortBy::Cpu,
        };
        Self {
            sys: System::new_all(),
            last_updated: Instant::now(),
            refresh_rate: Duration::from_millis(config.refresh_rate),
            search_query: String::new(),
            searching: false,
            sort_by,
            descending: true,
            table_state: TableState::default(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config: Config = toml::from_str(&fs::read_to_string("config.toml").unwrap_or_default()).unwrap_or(Config {
        refresh_rate: 1000,
        default_sort: "cpu".into(),
    });

    let mut app = App::new(config);

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('/') => {
                        app.searching = true;
                        app.search_query.clear();
                    },
                    KeyCode::Esc => {
                        app.searching = false;
                        app.search_query.clear();
                    },
                    KeyCode::Char(c) if app.searching => app.search_query.push(c),
                    KeyCode::Backspace if app.searching => { app.search_query.pop(); },
                    KeyCode::Char('c') => app.sort_by = SortBy::Cpu,
                    KeyCode::Char('m') => app.sort_by = SortBy::Mem,
                    KeyCode::Char('n') => app.sort_by = SortBy::Name,
                    KeyCode::Char('r') => app.descending = !app.descending,
                    KeyCode::Up => move_selection(&mut app, -1),
                    KeyCode::Down => move_selection(&mut app, 1),
                    _ => {},
                }
            }
        }

        if app.last_updated.elapsed() >= app.refresh_rate {
            app.sys.refresh_all();
            app.last_updated = Instant::now();
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            // Search bar
            let search = Paragraph::new(if app.searching {
                format!("Search: {}", app.search_query)
            } else {
                "Press '/' to search, 'q' to quit".to_string()
            });
            f.render_widget(search, chunks[0]);

            // Network stats
            let net = app.sys.networks();
            let net_info = net.iter().map(|(iface, data)| {
                format!("{} ↓{} KB ↑{} KB", iface, data.total_received() / 1024, data.total_transmitted() / 1024)
            }).collect::<Vec<_>>().join(" | ");
            f.render_widget(Paragraph::new(net_info), chunks[1]);

            // Build owned process info vector
            let mut processes: Vec<ProcInfo> = app.sys.processes().values().map(|p| ProcInfo {
                pid: p.pid(),
                name: p.name().to_string(),
                cpu: p.cpu_usage(),
                mem: p.memory(),
            }).collect();

            // Apply search filter
            if !app.search_query.is_empty() {
                let query = app.search_query.to_lowercase();
                processes.retain(|p| p.name.to_lowercase().contains(&query) || p.pid.to_string().contains(&query));
            }

            // Sort processes using owned data
            sort_processes(&app, &mut processes);

            // Map to table rows
            let rows: Vec<Row> = processes.iter().map(|p| {
                Row::new(vec![
                    p.pid.to_string(),
                    p.name.clone(),
                    format!("{:.2}%", p.cpu),
                    format!("{:.2} MB", p.mem as f64 / 1024.0),
                ])
            }).collect();

            let table = Table::new(rows)
                .header(Row::new(vec!["PID", "Name", "CPU %", "Memory MB"]).style(Style::default().add_modifier(Modifier::BOLD)))
                .block(Block::default().borders(Borders::ALL).title("Processes"))
                .widths(&[
                    Constraint::Length(10),
                    Constraint::Length(25),
                    Constraint::Length(10),
                    Constraint::Length(15),
                ])
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            f.render_stateful_widget(table, chunks[2], &mut app.table_state);
        })?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn sort_processes(app: &App, processes: &mut Vec<ProcInfo>) {
    match app.sort_by {
        SortBy::Cpu => processes.sort_by(|a, b| a.cpu.partial_cmp(&b.cpu).unwrap()),
        SortBy::Mem => processes.sort_by(|a, b| a.mem.cmp(&b.mem)),
        SortBy::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
    }
    if app.descending {
        processes.reverse();
    }
}

fn move_selection(app: &mut App, delta: isize) {
    let i = app.table_state.selected().unwrap_or(0) as isize + delta;
    let len = app.sys.processes().len() as isize;
    app.table_state.select(Some((i.max(0).min(len - 1)) as usize));
}
