use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::app::{App, AppState, MenuItem};
use crate::puzzle::CellState;

// Color palette for the solved reveal (cycles by diagonal)
const REVEAL_PALETTE: [Color; 8] = [
    Color::Cyan,
    Color::LightCyan,
    Color::LightBlue,
    Color::Blue,
    Color::LightMagenta,
    Color::Magenta,
    Color::LightGreen,
    Color::Green,
];

fn fmt_ms(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let s = secs % 60;
    format!("{}:{:02}", mins, s)
}

pub fn render(f: &mut Frame, app: &App) {
    match app.state {
        AppState::Splash => render_splash(f),
        AppState::Menu => render_menu(f, app),
        AppState::Playing => render_playing(f, app),
        AppState::Solved => render_solved(f, app),
        AppState::Quit => {}
    }
}

// ── Splash ───────────────────────────────────────────────────────────────────

fn render_splash(f: &mut Frame) {
    let area = f.area();

    let art = vec![
        "",
        "  ██████╗ ██╗ ██████╗██████╗  ██████╗ ███████╗███████╗██╗  ██╗",
        "  ██╔══██╗██║██╔════╝██╔══██╗██╔═══██╗██╔════╝██╔════╝██║  ██║",
        "  ██████╔╝██║██║     ██████╔╝██║   ██║███████╗███████╗███████║",
        "  ██╔═══╝ ██║██║     ██╔══██╗██║   ██║╚════██║╚════██║██╔══██║",
        "  ██║     ██║╚██████╗██║  ██║╚██████╔╝███████║███████║██║  ██║",
        "  ╚═╝     ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝╚═╝  ╚═╝",
        "",
        "          a terminal nonogram puzzle game",
        "",
        "",
        "              ┌────────────────────────────┐",
        "              │   Press any key to start   │",
        "              └────────────────────────────┘",
        "",
    ];

    let art_lines: Vec<Line> = art
        .iter()
        .enumerate()
        .map(|(i, &line)| {
            if i >= 1 && i <= 6 {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ))
            } else if i == 8 {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::White).add_modifier(Modifier::DIM),
                ))
            } else if i >= 11 && i <= 13 {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(line)
            }
        })
        .collect();

    let art_height = art_lines.len() as u16;
    let art_width = art.iter().map(|l| l.chars().count()).max().unwrap_or(60) as u16;

    let v_offset = area.top() + area.height.saturating_sub(art_height) / 2;
    let h_offset = area.left() + area.width.saturating_sub(art_width) / 2;

    let splash_area = Rect {
        x: h_offset,
        y: v_offset,
        width: art_width.min(area.width),
        height: art_height.min(area.height),
    };

    f.render_widget(Paragraph::new(Text::from(art_lines)), splash_area);
}

// ── Menu ─────────────────────────────────────────────────────────────────────

fn render_menu(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Paragraph::new(" ✦  PICROSSH  ✦ ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Blue)),
        );
    f.render_widget(header, chunks[0]);

    // Determine column width for name so sizes line up in a second column.
    // Use max of all display names (hidden or revealed) plus a small margin.
    let name_col_w = app
        .menu_items
        .iter()
        .filter_map(|item| {
            if let MenuItem::PuzzleEntry(idx) = item {
                Some(app.puzzle_display_name(*idx).chars().count())
            } else {
                None
            }
        })
        .max()
        .unwrap_or(8)
        + 2; // a little breathing room

    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .map(|item| match item {
            MenuItem::Header(label) => ListItem::new(Line::from(Span::styled(
                format!("  {}", label),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))),
            MenuItem::PuzzleEntry(idx) => {
                let p = &app.puzzles[*idx];
                let solved = app.solved_puzzles.contains(idx);
                let name = app.puzzle_display_name(*idx);
                let size_str = format!("{}×{}", p.rows, p.cols);
                let (prefix, name_style, size_style) = if solved {
                    (
                        "  ✓ ",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::DarkGray),
                    )
                } else {
                    (
                        "    ",
                        Style::default().fg(Color::White),
                        Style::default().fg(Color::DarkGray),
                    )
                };
                let mut spans = vec![
                    Span::raw(prefix),
                    Span::styled(
                        format!("{:<width$}", name, width = name_col_w),
                        name_style,
                    ),
                    Span::styled(size_str, size_style),
                ];
                if let Some(ms) = app.best_time_ms(*idx) {
                    spans.push(Span::styled(
                        format!("  {}", fmt_ms(ms)),
                        Style::default().fg(Color::Yellow),
                    ));
                }
                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.menu_selection));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " Select Puzzle ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let list_area = centered_rect(50, 80, chunks[1]);
    f.render_stateful_widget(list, list_area, &mut state);

    let footer = Paragraph::new("↑↓ / jk = navigate   Enter / Space = play   q = quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

// ── Playing ───────────────────────────────────────────────────────────────────

fn render_playing(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let elapsed = app.live_elapsed();
    let display_name = app.puzzle_display_name(app.current_puzzle_index);

    let header_text = format!("PICROSSH   │   {}   │   {}", display_name, format_time(elapsed));
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Blue)),
        );
    f.render_widget(header, chunks[0]);

    let footer_text =
        "Space=fill  e=erase  x=cross  r=reset  hjkl/arrows=move  HJKL=×5  g/G=top/bot  0/$=row±  Esc=menu"
            .to_string();
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);

    render_grid(f, app, chunks[1]);
}

// ── Solved reveal ─────────────────────────────────────────────────────────────

fn render_solved(f: &mut Frame, app: &App) {
    let area = f.area();
    let puzzle = &app.puzzles[app.current_puzzle_index];

    // Header
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let display_name = puzzle.name;
    let header_text = format!(
        "PICROSSH   │   {}   │   {}",
        display_name,
        format_time(app.elapsed)
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Blue)),
        );
    f.render_widget(header, chunks[0]);

    // Banner at the bottom
    let banner = Paragraph::new(format!(
        " ✦  SOLVED!  ✦   {}   │   {}   │   n = next puzzle   Esc = menu   q = quit",
        display_name,
        format_time(app.elapsed),
    ))
    .style(Style::default().fg(Color::Black).bg(Color::LightGreen).add_modifier(Modifier::BOLD))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    );
    f.render_widget(banner, chunks[2]);

    // Reveal: show just the pixel art, no clues, no grid lines
    render_reveal(f, app, chunks[1]);
}

fn render_reveal(f: &mut Frame, app: &App, area: Rect) {
    let puzzle = &app.puzzles[app.current_puzzle_index];

    // Each cell is 2 wide, no separators between cells in reveal mode
    let cell_w: usize = 2;
    let grid_w = puzzle.cols * cell_w;
    let grid_h = puzzle.rows;

    if (area.width as usize) < grid_w || (area.height as usize) < grid_h {
        return;
    }

    let v_offset = area.top() + area.height.saturating_sub(grid_h as u16) / 2;
    let h_offset = area.left() + area.width.saturating_sub(grid_w as u16) / 2;

    let mut lines: Vec<Line> = Vec::new();
    for r in 0..puzzle.rows {
        let mut spans = Vec::new();
        for c in 0..puzzle.cols {
            if puzzle.solution[r][c] {
                let color = REVEAL_PALETTE[(r + c) % REVEAL_PALETTE.len()];
                // Use gradient shading: cells near edges use lighter blocks
                let dist_from_edge = (r.min(puzzle.rows - 1 - r)).min(c.min(puzzle.cols - 1 - c));
                let block_char = match dist_from_edge {
                    0 => "░░",
                    1 => "▒▒",
                    2 => "▓▓",
                    _ => "██",
                };
                spans.push(Span::styled(block_char, Style::default().fg(color)));
            } else {
                spans.push(Span::raw("  "));
            }
        }
        lines.push(Line::from(spans));
    }

    let reveal_area = Rect {
        x: h_offset,
        y: v_offset,
        width: (grid_w as u16).min(area.width),
        height: (grid_h as u16).min(area.height),
    };
    f.render_widget(Paragraph::new(Text::from(lines)), reveal_area);
}

// ── Grid ──────────────────────────────────────────────────────────────────────

fn clue_satisfied(board_line: &[CellState], clue: &[usize]) -> bool {
    let mut runs = vec![];
    let mut count = 0;
    for &cell in board_line {
        if cell == CellState::Filled {
            count += 1;
        } else if count > 0 {
            runs.push(count);
            count = 0;
        }
    }
    if count > 0 {
        runs.push(count);
    }
    if runs.is_empty() {
        runs.push(0);
    }
    runs == clue
}

fn render_grid(f: &mut Frame, app: &App, area: Rect) {
    let puzzle = &app.puzzles[app.current_puzzle_index];

    let row_clue_width = puzzle
        .row_clues
        .iter()
        .map(|clues| clues.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ").len())
        .max()
        .unwrap_or(1);

    let col_clue_height = puzzle.col_clues.iter().map(|c| c.len()).max().unwrap_or(1);

    // Each cell is 2 chars; between cells: │ (1 char); plus │ on each side → total cols*3+1
    // Row: row_clue_width + " " + │ + cells = row_clue_width + 1 + cols*3 + 1 = row_clue_width + cols*3 + 2
    let needed_width = row_clue_width + puzzle.cols * 3 + 2;
    // Height: col clue rows + separator + top border + data rows + separator rows + bottom border
    // = col_clue_height + 1 + 1 + rows*2 - 1 + 1 = col_clue_height + rows*2 + 2
    let needed_height = col_clue_height + puzzle.rows * 2 + 2;

    if (area.width as usize) < needed_width || (area.height as usize) < needed_height {
        let msg = Paragraph::new("Terminal too small — please resize")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        f.render_widget(msg, area);
        return;
    }

    let border_style = Style::default().fg(Color::Blue);
    let sep_style = Style::default().fg(Color::DarkGray);
    // pad aligns col clue area: row_clue_width spaces + 1 space (to sit above " │" prefix)
    let pad = " ".repeat(row_clue_width + 1);

    let mut lines: Vec<Line> = Vec::new();

    // Column clue rows
    for d in 0..col_clue_height {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(pad.clone())); // row_clue_width + 1 spaces
        spans.push(Span::raw(" "));         // 1 more: total row_clue_width+2 before first clue,
                                             // which aligns with row_clue_width + " " + "│" in grid rows
        for c in 0..puzzle.cols {
            let clues = &puzzle.col_clues[c];
            let offset = col_clue_height - clues.len();
            let satisfied = !app.board.is_empty() && {
                let col_cells: Vec<CellState> = (0..puzzle.rows).map(|r| app.board[r][c]).collect();
                clue_satisfied(&col_cells, clues)
            };
            let cell_str = if d >= offset {
                format!("{:>2}", clues[d - offset])
            } else {
                "  ".to_string()
            };
            let clue_style = if satisfied {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(cell_str, clue_style));
            if c + 1 < puzzle.cols {
                spans.push(Span::styled("│", sep_style));
            }
        }
        lines.push(Line::from(spans));
    }

    // Separator between col clues and top border
    // "─┬" puts ─ at row_clue_width, ┬ at row_clue_width+1, aligned with grid's left │
    {
        let mut spans = vec![Span::raw(" ".repeat(row_clue_width))];
        spans.push(Span::styled("─┬", sep_style));
        for c in 0..puzzle.cols {
            spans.push(Span::styled("──", sep_style));
            if c + 1 < puzzle.cols {
                spans.push(Span::styled("┬", sep_style));
            }
        }
        spans.push(Span::styled("─", sep_style));
        lines.push(Line::from(spans));
    }

    // Top border of grid
    {
        let mut spans = vec![Span::raw(" ".repeat(row_clue_width + 1))];
        spans.push(Span::styled("┌", border_style));
        for c in 0..puzzle.cols {
            spans.push(Span::styled("──", border_style));
            if c + 1 < puzzle.cols {
                spans.push(Span::styled("┬", border_style));
            }
        }
        spans.push(Span::styled("┐", border_style));
        lines.push(Line::from(spans));
    }

    // Grid rows
    for r in 0..puzzle.rows {
        // Data row
        {
            let mut spans: Vec<Span> = Vec::new();

            let clue_str = puzzle.row_clues[r]
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let row_satisfied =
                !app.board.is_empty() && clue_satisfied(&app.board[r], &puzzle.row_clues[r]);
            let clue_style = if row_satisfied {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(
                format!("{:>width$}", clue_str, width = row_clue_width),
                clue_style,
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled("│", border_style));

            for c in 0..puzzle.cols {
                let is_cursor = r == app.cursor_row && c == app.cursor_col;
                let cell = app.board[r][c];

                let (text, style) = match (cell, is_cursor) {
                    (CellState::Empty, false) => ("  ", Style::default()),
                    (CellState::Empty, true) => {
                        ("  ", Style::default().bg(Color::Yellow).fg(Color::Black))
                    }
                    (CellState::Filled, false) => ("██", Style::default().fg(Color::Cyan)),
                    (CellState::Filled, true) => {
                        ("██", Style::default().fg(Color::White).bg(Color::Cyan))
                    }
                    (CellState::Crossed, false) => {
                        ("╳╳", Style::default().fg(Color::DarkGray))
                    }
                    (CellState::Crossed, true) => {
                        ("╳╳", Style::default().fg(Color::White).bg(Color::DarkGray))
                    }
                };

                spans.push(Span::styled(text, style));
                if c + 1 < puzzle.cols {
                    spans.push(Span::styled("│", border_style));
                }
            }
            spans.push(Span::styled("│", border_style));
            lines.push(Line::from(spans));
        }

        // Row separator (between rows, not after the last)
        if r + 1 < puzzle.rows {
            let mut spans = vec![Span::raw(" ".repeat(row_clue_width + 1))];
            spans.push(Span::styled("├", border_style));
            for c in 0..puzzle.cols {
                spans.push(Span::styled("──", border_style));
                if c + 1 < puzzle.cols {
                    spans.push(Span::styled("┼", border_style));
                }
            }
            spans.push(Span::styled("┤", border_style));
            lines.push(Line::from(spans));
        }
    }

    // Bottom border
    {
        let mut spans = vec![Span::raw(" ".repeat(row_clue_width + 1))];
        spans.push(Span::styled("└", border_style));
        for c in 0..puzzle.cols {
            spans.push(Span::styled("──", border_style));
            if c + 1 < puzzle.cols {
                spans.push(Span::styled("┴", border_style));
            }
        }
        spans.push(Span::styled("┘", border_style));
        lines.push(Line::from(spans));
    }

    let grid_height = lines.len() as u16;
    let grid_width = needed_width as u16;

    let v_offset = area.top() + area.height.saturating_sub(grid_height) / 2;
    let h_offset = area.left() + area.width.saturating_sub(grid_width) / 2;

    let grid_area = Rect {
        x: h_offset,
        y: v_offset,
        width: grid_width.min(area.width),
        height: grid_height.min(area.height),
    };

    f.render_widget(Paragraph::new(Text::from(lines)), grid_area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn format_time(d: Duration) -> String {
    let total = d.as_secs();
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
