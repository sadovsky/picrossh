use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::app::{App, AppState};
use crate::puzzle::CellState;

pub fn render(f: &mut Frame, app: &App) {
    match app.state {
        AppState::Menu => render_menu(f, app),
        AppState::Playing => render_playing(f, app),
        AppState::Solved => {
            render_playing(f, app);
            render_solved_overlay(f, app);
        }
        AppState::Quit => {}
    }
}

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

    let header = Paragraph::new("PICROSSH")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = app
        .puzzles
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let label = format!("  {}. {} ({}×{})", i + 1, p.name, p.rows, p.cols);
            ListItem::new(label)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.menu_selection));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Puzzle "))
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
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn render_playing(f: &mut Frame, app: &App) {
    let area = f.area();
    let puzzle = &app.puzzles[app.current_puzzle_index];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let elapsed = if app.state == AppState::Solved { app.elapsed } else { app.live_elapsed() };

    let header_text = format!("PICROSSH   │   {}   │   {}", puzzle.name, format_time(elapsed));
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let footer_text = if app.state == AppState::Solved {
        "SOLVED!   Esc = menu   n = next puzzle   q = quit"
    } else {
        "Space = fill   x = cross   r = reset   Esc = menu   q = quit"
    };
    let footer = Paragraph::new(footer_text).alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);

    render_grid(f, app, chunks[1]);
}

fn render_grid(f: &mut Frame, app: &App, area: Rect) {
    let puzzle = &app.puzzles[app.current_puzzle_index];

    let row_clue_width = puzzle
        .row_clues
        .iter()
        .map(|clues| {
            clues.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ").len()
        })
        .max()
        .unwrap_or(1);

    let col_clue_height = puzzle.col_clues.iter().map(|c| c.len()).max().unwrap_or(1);

    let cell_area_width = puzzle.cols * 3 - puzzle.cols.min(1);
    let needed_width = row_clue_width + 3 + cell_area_width;
    let needed_height = col_clue_height + 1 + puzzle.rows;

    if (area.width as usize) < needed_width || (area.height as usize) < needed_height {
        let msg = Paragraph::new("Terminal too small — please resize")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        f.render_widget(msg, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Column clue rows
    for d in 0..col_clue_height {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" ".repeat(row_clue_width + 3)));
        for c in 0..puzzle.cols {
            let clues = &puzzle.col_clues[c];
            let offset = col_clue_height - clues.len();
            let cell_str = if d >= offset {
                format!("{:>2}", clues[d - offset])
            } else {
                "  ".to_string()
            };
            spans.push(Span::raw(cell_str));
            if c + 1 < puzzle.cols {
                spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(spans));
    }

    // Separator
    let sep = format!("{}-+-{}", " ".repeat(row_clue_width), "-".repeat(cell_area_width));
    lines.push(Line::raw(sep));

    // Grid rows
    for r in 0..puzzle.rows {
        let mut spans: Vec<Span> = Vec::new();

        let clue_str = puzzle.row_clues[r]
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        spans.push(Span::raw(format!("{:>width$}", clue_str, width = row_clue_width)));
        spans.push(Span::raw(" | "));

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
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    let grid_height = (col_clue_height + 1 + puzzle.rows) as u16;
    let grid_width = (row_clue_width + 3 + cell_area_width) as u16;

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

fn render_solved_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let overlay = centered_rect(40, 35, area);

    let content = format!("\n  SOLVED!\n\n  Time: {}\n", format_time(app.elapsed));
    let block = Paragraph::new(content)
        .style(Style::default().fg(Color::Black).bg(Color::Green))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Black).bg(Color::Green))
                .title(" Congratulations! "),
        );

    f.render_widget(Clear, overlay);
    f.render_widget(block, overlay);
}

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
