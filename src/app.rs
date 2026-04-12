use std::time::{Duration, Instant};

use crate::puzzle::{derive_clues, CellState, Puzzle};

#[derive(PartialEq, Eq)]
pub enum AppState {
    Menu,
    Playing,
    Solved,
    Quit,
}

pub struct App {
    pub state: AppState,
    pub puzzles: Vec<Puzzle>,
    pub current_puzzle_index: usize,
    pub board: Vec<Vec<CellState>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub start_time: Option<Instant>,
    pub elapsed: Duration,
    pub menu_selection: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            state: AppState::Menu,
            puzzles: Puzzle::presets(),
            current_puzzle_index: 0,
            board: vec![],
            cursor_row: 0,
            cursor_col: 0,
            start_time: None,
            elapsed: Duration::ZERO,
            menu_selection: 0,
        }
    }

    /// Start directly in Playing state with a single puzzle (no menu).
    pub fn with_puzzle(puzzle: Puzzle) -> Self {
        let mut app = App {
            state: AppState::Menu,
            puzzles: vec![puzzle],
            current_puzzle_index: 0,
            board: vec![],
            cursor_row: 0,
            cursor_col: 0,
            start_time: None,
            elapsed: Duration::ZERO,
            menu_selection: 0,
        };
        app.load_puzzle(0);
        app
    }

    pub fn load_puzzle(&mut self, index: usize) {
        self.current_puzzle_index = index;
        let puzzle = &self.puzzles[index];
        self.board = vec![vec![CellState::Empty; puzzle.cols]; puzzle.rows];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.start_time = Some(Instant::now());
        self.elapsed = Duration::ZERO;
        self.state = AppState::Playing;
    }

    pub fn toggle_fill(&mut self) {
        let cell = &mut self.board[self.cursor_row][self.cursor_col];
        *cell = if *cell == CellState::Filled { CellState::Empty } else { CellState::Filled };
    }

    pub fn toggle_cross(&mut self) {
        let cell = &mut self.board[self.cursor_row][self.cursor_col];
        *cell = if *cell == CellState::Crossed { CellState::Empty } else { CellState::Crossed };
    }

    pub fn move_cursor(&mut self, dr: i32, dc: i32) {
        let puzzle = &self.puzzles[self.current_puzzle_index];
        self.cursor_row =
            (self.cursor_row as i32 + dr).clamp(0, puzzle.rows as i32 - 1) as usize;
        self.cursor_col =
            (self.cursor_col as i32 + dc).clamp(0, puzzle.cols as i32 - 1) as usize;
    }

    /// Verify all row and column clues match the current board.
    /// This supports any valid solution, not just the original one.
    pub fn check_solved(&mut self) {
        let puzzle = &self.puzzles[self.current_puzzle_index];

        for r in 0..puzzle.rows {
            let row: Vec<bool> =
                (0..puzzle.cols).map(|c| self.board[r][c] == CellState::Filled).collect();
            if derive_clues(&row) != puzzle.row_clues[r] {
                return;
            }
        }
        for c in 0..puzzle.cols {
            let col: Vec<bool> =
                (0..puzzle.rows).map(|r| self.board[r][c] == CellState::Filled).collect();
            if derive_clues(&col) != puzzle.col_clues[c] {
                return;
            }
        }

        if let Some(start) = self.start_time {
            self.elapsed = start.elapsed();
        }
        self.state = AppState::Solved;
    }

    pub fn reset_board(&mut self) {
        let puzzle = &self.puzzles[self.current_puzzle_index];
        self.board = vec![vec![CellState::Empty; puzzle.cols]; puzzle.rows];
    }

    pub fn live_elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |s| s.elapsed())
    }

    pub fn menu_up(&mut self) {
        if self.menu_selection > 0 {
            self.menu_selection -= 1;
        }
    }

    pub fn menu_down(&mut self) {
        if self.menu_selection + 1 < self.puzzles.len() {
            self.menu_selection += 1;
        }
    }

    pub fn next_puzzle(&mut self) {
        let next = (self.current_puzzle_index + 1) % self.puzzles.len();
        self.load_puzzle(next);
    }
}
