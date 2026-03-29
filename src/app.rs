use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::puzzle::{CellState, Puzzle};

#[derive(PartialEq, Eq)]
pub enum AppState {
    Splash,
    Menu,
    Playing,
    Solved,
    Quit,
}

#[derive(Clone)]
pub enum MenuItem {
    Header(String),
    PuzzleEntry(usize),
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
    pub solved_puzzles: HashSet<usize>,
    pub menu_items: Vec<MenuItem>,
    pub paint_mode: bool,
    pub paint_value: CellState,
}

fn build_menu_items(puzzles: &[Puzzle]) -> Vec<MenuItem> {
    let mut items = vec![];
    let mut last_size: Option<(usize, usize)> = None;
    for (i, p) in puzzles.iter().enumerate() {
        let size = (p.rows, p.cols);
        if Some(size) != last_size {
            items.push(MenuItem::Header(format!("── {}×{} ──", p.rows, p.cols)));
            last_size = Some(size);
        }
        items.push(MenuItem::PuzzleEntry(i));
    }
    items
}

fn first_puzzle_entry(items: &[MenuItem]) -> usize {
    items.iter().position(|item| matches!(item, MenuItem::PuzzleEntry(_))).unwrap_or(0)
}

impl App {
    pub fn new() -> Self {
        let puzzles = Puzzle::presets();
        let menu_items = build_menu_items(&puzzles);
        let initial_selection = first_puzzle_entry(&menu_items);
        App {
            state: AppState::Splash,
            puzzles,
            current_puzzle_index: 0,
            board: vec![],
            cursor_row: 0,
            cursor_col: 0,
            start_time: None,
            elapsed: Duration::ZERO,
            menu_selection: initial_selection,
            solved_puzzles: HashSet::new(),
            menu_items,
            paint_mode: false,
            paint_value: CellState::Empty,
        }
    }

    pub fn load_puzzle(&mut self, index: usize) {
        self.current_puzzle_index = index;
        let puzzle = &self.puzzles[index];
        self.board = vec![vec![CellState::Empty; puzzle.cols]; puzzle.rows];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.start_time = Some(Instant::now());
        self.elapsed = Duration::ZERO;
        self.paint_mode = false;
        self.state = AppState::Playing;
    }

    pub fn toggle_fill(&mut self) {
        let cell = &mut self.board[self.cursor_row][self.cursor_col];
        *cell = if *cell == CellState::Filled { CellState::Empty } else { CellState::Filled };
        // Enter paint mode, painting whatever state we just set
        self.paint_value = self.board[self.cursor_row][self.cursor_col];
        self.paint_mode = true;
    }

    pub fn stop_paint(&mut self) {
        self.paint_mode = false;
    }

    /// Move cursor and, if in paint mode, apply paint to the new cell.
    pub fn move_and_paint(&mut self, dr: i32, dc: i32) {
        self.move_cursor(dr, dc);
        if self.paint_mode {
            self.board[self.cursor_row][self.cursor_col] = self.paint_value;
        }
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

    pub fn check_solved(&mut self) {
        let puzzle = &self.puzzles[self.current_puzzle_index];
        for r in 0..puzzle.rows {
            for c in 0..puzzle.cols {
                let should_fill = puzzle.solution[r][c];
                let is_filled = self.board[r][c] == CellState::Filled;
                if should_fill != is_filled {
                    return;
                }
            }
        }
        if let Some(start) = self.start_time {
            self.elapsed = start.elapsed();
        }
        self.solved_puzzles.insert(self.current_puzzle_index);
        self.state = AppState::Solved;
    }

    pub fn reset_board(&mut self) {
        let puzzle = &self.puzzles[self.current_puzzle_index];
        self.board = vec![vec![CellState::Empty; puzzle.cols]; puzzle.rows];
    }

    pub fn live_elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |s| s.elapsed())
    }

    pub fn puzzle_display_name(&self, index: usize) -> String {
        if self.solved_puzzles.contains(&index) {
            self.puzzles[index].name.to_string()
        } else {
            format!("Puzzle {}", index + 1)
        }
    }

    pub fn menu_up(&mut self) {
        if self.menu_selection == 0 {
            return;
        }
        let mut prev = self.menu_selection - 1;
        loop {
            if matches!(self.menu_items[prev], MenuItem::PuzzleEntry(_)) {
                self.menu_selection = prev;
                return;
            }
            if prev == 0 {
                return;
            }
            prev -= 1;
        }
    }

    pub fn menu_down(&mut self) {
        let mut next = self.menu_selection + 1;
        while next < self.menu_items.len() {
            if matches!(self.menu_items[next], MenuItem::PuzzleEntry(_)) {
                self.menu_selection = next;
                return;
            }
            next += 1;
        }
    }

    pub fn next_puzzle(&mut self) {
        let next = (self.current_puzzle_index + 1) % self.puzzles.len();
        self.load_puzzle(next);
    }
}
