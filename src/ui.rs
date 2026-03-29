use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::app::{App, AppState, MenuItem};
use crate::puzzle::CellState;

pub fn render(f: &mut Frame, app: &App) {
    match app.state {
        AppState::Splash => render_splash(f),
        AppState::Menu => render_menu(f, app),
        AppState::Playing => render_playing(f, app),
        AppState::Solved => {
            render_playing(f, app);
            render_solved_overlay(f, app);
        }
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
        "              ┌─────────────────────────┐",
        "              │   Press any key to start  │",
        "              └─────────────────────────┘",
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
    let art_width = art
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(60) as u16;

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
                let label = if solved {
                    format!("  ✓ {} ({}×{})", name, p.rows, p.cols)
                } else {
                    format!("    {} ({}×{})", name, p.rows, p.cols)
                };
                let style = if solved {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(label, style)))
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

    let footer =
        Paragraph::new("↑↓ / jk = navigate   Enter / Space = play   q = quit")
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

    let elapsed = if app.state == AppState::Solved { app.elapsed } else { app.live_elapsed() };
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

    let footer_text = if app.state == AppState::Solved {
        "SOLVED!   Esc = menu   n = next   q = quit"
    } else {
        "Space = fill   x = cross   r = reset   hjkl / arrows = move   g/G = top/bot   Esc = menu"
    };
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);

    render_grid(f, app, chunks[1]);
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

    // Grid interior: cols*2 wide with │ separators between = cols*3-1, plus │ on each side = cols*3+1
    // Total line: row_clue_width + 1 (space) + cols*3+1
    let grid_inner_width = puzzle.cols * 3 - 1; // cells + separators between
    let needed_width = row_clue_width + 2 + grid_inner_width + 2; // margin + "│" + cells + "│"
    // Height: col clue rows + separator + top border + rows + separator rows + bottom border
    // = col_clue_height + 1 + 1 + rows + (rows-1) + 1 = col_clue_height + rows*2 + 2
    let needed_height = col_clue_height + 1 + 1 + puzzle.rows * 2;

    if (area.width as usize) < needed_width || (area.height as usize) < needed_height {
        let msg = Paragraph::new("Terminal too small — please resize")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        f.render_widget(msg, area);
        return;
    }

    let border_style = Style::default().fg(Color::Blue);
    let sep_style = Style::default().fg(Color::DarkGray);
    let pad = " ".repeat(row_clue_width + 1); // aligns col clue area with grid

    let mut lines: Vec<Line> = Vec::new();

    // Column clue rows
    for d in 0..col_clue_height {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(pad.clone()));
        spans.push(Span::raw(" ")); // aligns with "│"
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
                spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(spans));
    }

    // Separator between clues and grid
    {
        let mut spans = vec![Span::raw(" ".repeat(row_clue_width))];
        spans.push(Span::styled(" ─┬", sep_style));
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
            let row_satisfied = !app.board.is_empty()
                && clue_satisfied(&app.board[r], &puzzle.row_clues[r]);
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
                    (CellState::Empty, true) => ("  ", Style::default().bg(Color::DarkGray)),
                    (CellState::Filled, false) => ("██", Style::default().fg(Color::Cyan)),
                    (CellState::Filled, true) => {
                        ("██", Style::default().fg(Color::White).bg(Color::Cyan))
                    }
                    (CellState::Crossed, false) => (" X", Style::default().fg(Color::DarkGray)),
                    (CellState::Crossed, true) => {
                        (" X", Style::default().fg(Color::White).bg(Color::DarkGray))
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

// ── Solved overlay ────────────────────────────────────────────────────────────

fn render_solved_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let overlay = centered_rect(40, 40, area);

    let puzzle_name = &app.puzzles[app.current_puzzle_index].name;
    let content = format!(
        "\n  ✦  SOLVED!  ✦\n\n  {}\n\n  Time: {}\n",
        puzzle_name,
        format_time(app.elapsed)
    );
    let block = Paragraph::new(content)
        .style(Style::default().fg(Color::Black).bg(Color::LightGreen))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().fg(Color::Black).bg(Color::LightGreen))
                .title(Span::styled(
                    " Congratulations! ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )),
        );

    f.render_widget(Clear, overlay);
    f.render_widget(block, overlay);
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
