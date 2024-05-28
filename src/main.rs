use rand::{thread_rng, Rng};
use sysinfo::System;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Sparkline},
    Frame, Terminal,
};

struct App {
    cpu: Vec<u64>,
    memory: Vec<u64>,
}

impl App {
    fn new() -> App {
        App {
            cpu: vec![100; 200],
            memory: vec![100; 200],
        }
    }

    fn next_cpu(&mut self, sys: &System) {
        let value = sys.global_cpu_info().cpu_usage() as u64;
        self.cpu.pop();
        self.cpu.insert(0, value);
    }

    fn next_memory(&mut self, sys: &System) {
        let value = (byte_to_megabyte(sys.used_memory()) as f64 / byte_to_megabyte(sys.total_memory()) as f64 * 100.0) as u64;
        self.memory.pop();
        self.memory.insert(0, value);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut sys = System::new_all();
    let tick_rate = Duration::from_millis(250);
    let res = run_app(&mut terminal, &mut sys, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    sys: &mut System,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let colors: [Color; 2] = [random_color(), random_color()];

    loop {
        terminal.draw(|f| ui(f, &app, &colors))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            sys.refresh_all();
            app.next_cpu(&sys);
            app.next_memory(&sys);
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App, colors: &[Color]) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Multiline
    let cpu_sparkline = Sparkline::default()
        .block(
            Block::default()
                .title("Cpu usage")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .data(&app.cpu)
        .style(Style::default().fg(colors[0]));
    f.render_widget(cpu_sparkline, chunks[0]);
    
    let memory_sparkline = Sparkline::default()
        .block(
            Block::default()
                .title("Memory usage")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .data(&app.memory)
        .style(Style::default().fg(colors[1]));
    f.render_widget(memory_sparkline, chunks[1]);
}

fn byte_to_megabyte(number: u64) -> u64 {
    number / 1024 / 1024
}

fn random_color() -> Color {
    let mut rng = thread_rng();
    Color::Rgb(rng.gen_range(1..=255), rng.gen_range(1..=255), rng.gen_range(1..=255))
}