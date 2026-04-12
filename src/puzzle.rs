#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Empty,
    Filled,
    Crossed,
}

pub struct Puzzle {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub solution: Option<Vec<Vec<bool>>>,
    pub row_clues: Vec<Vec<usize>>,
    pub col_clues: Vec<Vec<usize>>,
}

pub fn derive_clues(line: &[bool]) -> Vec<usize> {
    let mut clues = vec![];
    let mut count = 0;
    for &cell in line {
        if cell {
            count += 1;
        } else if count > 0 {
            clues.push(count);
            count = 0;
        }
    }
    if count > 0 {
        clues.push(count);
    }
    if clues.is_empty() {
        clues.push(0);
    }
    clues
}

impl Puzzle {
    pub fn new(name: impl Into<String>, solution: Vec<Vec<bool>>) -> Self {
        let rows = solution.len();
        let cols = if rows > 0 { solution[0].len() } else { 0 };

        let row_clues: Vec<Vec<usize>> = solution.iter().map(|row| derive_clues(row)).collect();

        let col_clues: Vec<Vec<usize>> = (0..cols)
            .map(|c| {
                let col: Vec<bool> = solution.iter().map(|row| row[c]).collect();
                derive_clues(&col)
            })
            .collect();

        Puzzle { name: name.into(), rows, cols, solution: Some(solution), row_clues, col_clues }
    }

    pub fn from_clues(
        name: String,
        rows: usize,
        cols: usize,
        row_clues: Vec<Vec<usize>>,
        col_clues: Vec<Vec<usize>>,
    ) -> Self {
        Puzzle { name, rows, cols, solution: None, row_clues, col_clues }
    }

    pub fn presets() -> Vec<Puzzle> {
        let f = false;
        let t = true;
        vec![
            // 5×5 Plus
            Puzzle::new(
                "Plus",
                vec![
                    vec![f, f, t, f, f],
                    vec![f, f, t, f, f],
                    vec![t, t, t, t, t],
                    vec![f, f, t, f, f],
                    vec![f, f, t, f, f],
                ],
            ),
            // 5×5 X Mark
            Puzzle::new(
                "X Mark",
                vec![
                    vec![t, f, f, f, t],
                    vec![f, t, f, t, f],
                    vec![f, f, t, f, f],
                    vec![f, t, f, t, f],
                    vec![t, f, f, f, t],
                ],
            ),
            // 10×10 House
            Puzzle::new(
                "House",
                vec![
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, t, t, t, t, f, f, f],
                    vec![f, f, t, t, t, t, t, t, f, f],
                    vec![f, t, t, t, t, t, t, t, t, f],
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, f, t, t, t, t, f, t, t],
                    vec![t, t, f, t, t, t, t, f, t, t],
                    vec![t, t, f, t, t, t, t, f, t, t],
                    vec![t, t, t, t, t, t, t, t, t, t],
                ],
            ),
            // 10×10 Heart
            Puzzle::new(
                "Heart",
                vec![
                    vec![f, t, t, f, f, f, t, t, f, f],
                    vec![t, t, t, t, f, t, t, t, t, f],
                    vec![t, t, t, t, t, t, t, t, t, f],
                    vec![t, t, t, t, t, t, t, t, t, f],
                    vec![f, t, t, t, t, t, t, t, f, f],
                    vec![f, f, t, t, t, t, t, f, f, f],
                    vec![f, f, f, t, t, t, f, f, f, f],
                    vec![f, f, f, f, t, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, f, f, f],
                ],
            ),
            // 10×10 Arrow
            Puzzle::new(
                "Arrow",
                vec![
                    vec![f, f, f, f, t, f, f, f, f, f],
                    vec![f, f, f, t, t, f, f, f, f, f],
                    vec![f, f, t, t, t, f, f, f, f, f],
                    vec![f, t, t, t, t, f, f, f, f, f],
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![f, t, t, t, t, f, f, f, f, f],
                    vec![f, f, t, t, t, f, f, f, f, f],
                    vec![f, f, f, t, t, f, f, f, f, f],
                    vec![f, f, f, f, t, f, f, f, f, f],
                ],
            ),
        ]
    }
}

// ─── .non format ───────────────────────────────────────────────────────────────

pub fn parse_non(input: &str) -> Result<Puzzle, String> {
    let mut name = "Untitled".to_string();
    let mut row_clues: Vec<Vec<usize>> = Vec::new();
    let mut col_clues: Vec<Vec<usize>> = Vec::new();
    let mut goal_rows: Vec<Vec<bool>> = Vec::new();

    #[derive(PartialEq)]
    enum State {
        Header,
        Rows,
        Columns,
        Goal,
    }
    let mut state = State::Header;

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("title") {
            let rest = rest.trim().trim_matches('"');
            name = rest.to_string();
            continue;
        }
        match line {
            "rows" => {
                state = State::Rows;
                continue;
            }
            "columns" => {
                state = State::Columns;
                continue;
            }
            "goal" => {
                state = State::Goal;
                continue;
            }
            _ => {}
        }
        match state {
            State::Header => {} // skip unknown header fields
            State::Rows => row_clues.push(parse_clue_line(line)?),
            State::Columns => col_clues.push(parse_clue_line(line)?),
            State::Goal => {
                let row: Vec<bool> = line
                    .chars()
                    .filter(|c| *c == '0' || *c == '1')
                    .map(|c| c == '1')
                    .collect();
                if row.is_empty() {
                    return Err(format!("Empty or invalid goal row: '{}'", line));
                }
                goal_rows.push(row);
            }
        }
    }

    if row_clues.is_empty() {
        return Err("No row clues found in puzzle file".to_string());
    }
    if col_clues.is_empty() {
        return Err("No column clues found in puzzle file".to_string());
    }

    let rows = row_clues.len();
    let cols = col_clues.len();

    let solution = if !goal_rows.is_empty() {
        if goal_rows.len() != rows {
            return Err(format!(
                "Goal has {} rows but puzzle has {} rows",
                goal_rows.len(),
                rows
            ));
        }
        for (i, row) in goal_rows.iter().enumerate() {
            if row.len() != cols {
                return Err(format!(
                    "Goal row {} has {} cells but puzzle has {} columns",
                    i,
                    row.len(),
                    cols
                ));
            }
        }
        Some(goal_rows)
    } else {
        None
    };

    let mut puzzle = Puzzle::from_clues(name, rows, cols, row_clues, col_clues);
    puzzle.solution = solution;
    Ok(puzzle)
}

fn parse_clue_line(line: &str) -> Result<Vec<usize>, String> {
    line.split(',')
        .map(|s| {
            s.trim()
                .parse::<usize>()
                .map_err(|_| format!("Invalid clue value: '{}'", s.trim()))
        })
        .collect()
}

pub fn write_non(puzzle: &Puzzle) -> String {
    let mut out = String::new();
    out.push_str(&format!("title \"{}\"\n", puzzle.name));
    out.push_str("\nrows\n");
    for clues in &puzzle.row_clues {
        let s = clues.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        out.push_str(&s);
        out.push('\n');
    }
    out.push_str("\ncolumns\n");
    for clues in &puzzle.col_clues {
        let s = clues.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        out.push_str(&s);
        out.push('\n');
    }
    if let Some(solution) = &puzzle.solution {
        out.push_str("\ngoal\n");
        for row in solution {
            let s: String = row.iter().map(|&b| if b { '1' } else { '0' }).collect();
            out.push_str(&s);
            out.push('\n');
        }
    }
    out
}
