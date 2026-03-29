use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod puzzle;
mod save;
mod ui;

use app::{App, AppState, MenuItem};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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

fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match app.state {
        AppState::Splash => {
            app.state = AppState::Menu;
        }
        AppState::Menu => match key {
            KeyCode::Char('q') => app.state = AppState::Quit,
            KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
            KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let MenuItem::PuzzleEntry(idx) = app.menu_items[app.menu_selection].clone() {
                    app.load_puzzle(idx);
                }
            }
            _ => {}
        },
        AppState::Playing => match key {
            KeyCode::Char('q') => app.state = AppState::Quit,
            KeyCode::Esc => {
                app.stop_paint();
                app.state = AppState::Menu;
            }
            // Shift+arrows: move 5 cells (must come before plain arrows)
            KeyCode::Up if modifiers.contains(KeyModifiers::SHIFT) => {
                app.move_and_paint(-5, 0);
                app.check_solved();
            }
            KeyCode::Down if modifiers.contains(KeyModifiers::SHIFT) => {
                app.move_and_paint(5, 0);
                app.check_solved();
            }
            KeyCode::Left if modifiers.contains(KeyModifiers::SHIFT) => {
                app.move_and_paint(0, -5);
                app.check_solved();
            }
            KeyCode::Right if modifiers.contains(KeyModifiers::SHIFT) => {
                app.move_and_paint(0, 5);
                app.check_solved();
            }
            // Plain arrows
            KeyCode::Up => {
                app.move_and_paint(-1, 0);
                app.check_solved();
            }
            KeyCode::Down => {
                app.move_and_paint(1, 0);
                app.check_solved();
            }
            KeyCode::Left => {
                app.move_and_paint(0, -1);
                app.check_solved();
            }
            KeyCode::Right => {
                app.move_and_paint(0, 1);
                app.check_solved();
            }
            // Vim movement
            KeyCode::Char('h') => {
                app.move_and_paint(0, -1);
                app.check_solved();
            }
            KeyCode::Char('j') => {
                app.move_and_paint(1, 0);
                app.check_solved();
            }
            KeyCode::Char('k') => {
                app.move_and_paint(-1, 0);
                app.check_solved();
            }
            KeyCode::Char('l') => {
                app.move_and_paint(0, 1);
                app.check_solved();
            }
            // Shift+hjkl: move 5 cells
            KeyCode::Char('H') => {
                app.move_and_paint(0, -5);
                app.check_solved();
            }
            KeyCode::Char('J') => {
                app.move_and_paint(5, 0);
                app.check_solved();
            }
            KeyCode::Char('K') => {
                app.move_and_paint(-5, 0);
                app.check_solved();
            }
            KeyCode::Char('L') => {
                app.move_and_paint(0, 5);
                app.check_solved();
            }
            // g/G: jump to top/bottom row (stops paint)
            KeyCode::Char('g') => {
                app.stop_paint();
                app.cursor_row = 0;
            }
            KeyCode::Char('G') => {
                app.stop_paint();
                app.cursor_row = app.puzzles[app.current_puzzle_index].rows - 1;
            }
            // 0/$: jump to start/end of row (stops paint)
            KeyCode::Char('0') => {
                app.stop_paint();
                app.cursor_col = 0;
            }
            KeyCode::Char('$') => {
                app.stop_paint();
                app.cursor_col = app.puzzles[app.current_puzzle_index].cols - 1;
            }
            KeyCode::Char(' ') => {
                app.fill_cell();
                app.check_solved();
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                app.erase_cell();
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                app.stop_paint();
                app.toggle_cross();
            }
            KeyCode::Char('r') => {
                app.stop_paint();
                app.reset_board();
            }
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
