use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod generate;
mod puzzle;
mod solver;
mod ui;

use app::{App, AppState};
use puzzle::{parse_non, write_non};

#[derive(Parser)]
#[command(name = "picrossh", about = "A TUI picross game")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Play a puzzle interactively.
    /// Reads a .non puzzle from stdin when piped; otherwise shows the built-in menu.
    Play,

    /// Generate a random NxN puzzle in .non format and write it to stdout.
    Generate {
        /// Side length of the square puzzle grid.
        #[arg(long, default_value_t = 10)]
        size: usize,
    },

    /// Solve a .non puzzle file and print the solution.
    Solve {
        /// Path to the .non puzzle file.
        file: PathBuf,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => run_tui(App::new()),
        Some(Command::Play) => run_play(),
        Some(Command::Generate { size }) => run_generate(size),
        Some(Command::Solve { file }) => run_solve(file),
    }
}

fn run_play() -> io::Result<()> {
    use std::io::IsTerminal;

    if io::stdin().is_terminal() {
        return run_tui(App::new());
    }

    // Stdin is a pipe: read the puzzle before crossterm takes over.
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Reopen the real terminal as fd 0 so crossterm can read key events.
    redirect_stdin_to_tty()?;

    let puzzle = parse_non(&input)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    run_tui(App::with_puzzle(puzzle))
}

fn run_generate(size: usize) -> io::Result<()> {
    let puzzle = generate::generate(size);
    print!("{}", write_non(&puzzle));
    Ok(())
}

fn run_solve(file: PathBuf) -> io::Result<()> {
    let input = std::fs::read_to_string(&file)?;
    let puzzle = parse_non(&input)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    match solver::solve(&puzzle) {
        Some(solution) => {
            println!("Solution for \"{}\" ({}×{}):", puzzle.name, puzzle.rows, puzzle.cols);
            for row in &solution {
                let line: String =
                    row.iter().map(|&b| if b { '#' } else { '.' }).collect();
                println!("{}", line);
            }
        }
        None => {
            eprintln!("No solution found for \"{}\"", puzzle.name);
            std::process::exit(1);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn redirect_stdin_to_tty() -> io::Result<()> {
    unsafe {
        let tty =
            libc::open(b"/dev/tty\0".as_ptr() as *const libc::c_char, libc::O_RDWR);
        if tty < 0 {
            return Err(io::Error::last_os_error());
        }
        libc::dup2(tty, 0);
        libc::close(tty);
    }
    Ok(())
}

#[cfg(not(unix))]
fn redirect_stdin_to_tty() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Piped stdin is not supported on this platform; use a file argument instead",
    ))
}

fn run_tui(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers);
            }
        }

        if app.state == AppState::Quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) {
    match app.state {
        AppState::Menu => match key {
            KeyCode::Char('q') => app.state = AppState::Quit,
            KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
            KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let idx = app.menu_selection;
                app.load_puzzle(idx);
            }
            _ => {}
        },
        AppState::Playing => match key {
            KeyCode::Char('q') => app.state = AppState::Quit,
            KeyCode::Esc => app.state = AppState::Menu,
            KeyCode::Up => app.move_cursor(-1, 0),
            KeyCode::Down => app.move_cursor(1, 0),
            KeyCode::Left => app.move_cursor(0, -1),
            KeyCode::Right => app.move_cursor(0, 1),
            KeyCode::Char(' ') => {
                app.toggle_fill();
                app.check_solved();
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                app.toggle_cross();
            }
            KeyCode::Char('r') => app.reset_board(),
            _ => {}
        },
        AppState::Solved => match key {
            KeyCode::Char('q') => app.state = AppState::Quit,
            KeyCode::Esc => app.state = AppState::Menu,
            KeyCode::Char('n') => app.next_puzzle(),
            _ => {}
        },
        AppState::Quit => {}
    }
}
