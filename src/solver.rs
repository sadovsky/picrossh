use crate::puzzle::Puzzle;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown,
    Filled,
    Empty,
}

// Enumerate all valid placements of `clues[clue_idx..]` into `line[pos..]`,
// accumulating per-cell fill counts and total placement count.
fn enum_placements(
    clues: &[usize],
    clue_idx: usize,
    pos: usize,
    line: &[Cell],
    current: &mut Vec<bool>,
    fill_count: &mut Vec<usize>,
    total: &mut usize,
) {
    let n = line.len();

    if clue_idx == clues.len() {
        // Valid if no Filled cells remain uncovered after the last block.
        // pos can be n+1 when the last block ends exactly at the line boundary,
        // so cap it to avoid an out-of-bounds slice.
        if line[pos.min(n)..].iter().any(|&c| c == Cell::Filled) {
            return;
        }
        *total += 1;
        for (i, &filled) in current.iter().enumerate() {
            if filled {
                fill_count[i] += 1;
            }
        }
        return;
    }

    let clue = clues[clue_idx];
    // Minimum space needed from pos onward for all remaining blocks + mandatory gaps.
    let remaining: usize =
        clues[clue_idx..].iter().sum::<usize>() + clues[clue_idx..].len().saturating_sub(1);
    if n < remaining || pos > n - remaining {
        return;
    }
    let max_start = n - remaining;

    for start in pos..=max_start {
        // Cannot skip over a Filled cell — it would remain uncovered.
        if line[pos..start].iter().any(|&c| c == Cell::Filled) {
            break;
        }
        // The block's span must contain no Empty cells.
        if line[start..start + clue].iter().any(|&c| c == Cell::Empty) {
            continue;
        }
        // The cell immediately after the block must not be Filled (gap required).
        if start + clue < n && line[start + clue] == Cell::Filled {
            continue;
        }

        for i in start..start + clue {
            current[i] = true;
        }
        enum_placements(
            clues,
            clue_idx + 1,
            start + clue + 1,
            line,
            current,
            fill_count,
            total,
        );
        for i in start..start + clue {
            current[i] = false;
        }
    }
}

// For a single line, determine which cells are forced Filled or Empty across
// all valid placements. Returns None on contradiction.
fn solve_line(clues: &[usize], line: &[Cell]) -> Option<Vec<Cell>> {
    let n = line.len();

    // Clue [0] means the entire line must be empty.
    if clues.len() == 1 && clues[0] == 0 {
        for &c in line {
            if c == Cell::Filled {
                return None;
            }
        }
        return Some(vec![Cell::Empty; n]);
    }

    let mut fill_count = vec![0usize; n];
    let mut total = 0usize;
    let mut current = vec![false; n];

    enum_placements(clues, 0, 0, line, &mut current, &mut fill_count, &mut total);

    if total == 0 {
        return None; // contradiction
    }

    let mut result = line.to_vec();
    for i in 0..n {
        if fill_count[i] == total {
            result[i] = Cell::Filled;
        } else if fill_count[i] == 0 {
            result[i] = Cell::Empty;
        }
    }
    Some(result)
}

// One full pass over all rows and columns.
// Returns Ok(true) if anything changed, Ok(false) if stable, Err(()) on contradiction.
fn propagate(
    grid: &mut Vec<Vec<Cell>>,
    row_clues: &[Vec<usize>],
    col_clues: &[Vec<usize>],
) -> Result<bool, ()> {
    let rows = row_clues.len();
    let cols = col_clues.len();
    let mut changed = false;

    for r in 0..rows {
        let line: Vec<Cell> = grid[r].clone();
        let new_line = solve_line(&row_clues[r], &line).ok_or(())?;
        for c in 0..cols {
            if new_line[c] != Cell::Unknown && grid[r][c] != new_line[c] {
                grid[r][c] = new_line[c];
                changed = true;
            }
        }
    }

    for c in 0..cols {
        let line: Vec<Cell> = (0..rows).map(|r| grid[r][c]).collect();
        let new_line = solve_line(&col_clues[c], &line).ok_or(())?;
        for r in 0..rows {
            if new_line[r] != Cell::Unknown && grid[r][c] != new_line[r] {
                grid[r][c] = new_line[r];
                changed = true;
            }
        }
    }

    Ok(changed)
}

// Propagate to a fixed point, then backtrack on the first unknown cell.
fn backtrack(
    grid: &mut Vec<Vec<Cell>>,
    row_clues: &[Vec<usize>],
    col_clues: &[Vec<usize>],
) -> bool {
    // Run propagation to a fixed point.
    loop {
        match propagate(grid, row_clues, col_clues) {
            Err(()) => return false,
            Ok(false) => break,
            Ok(true) => {}
        }
    }

    // Find first unknown cell.
    let mut unknown = None;
    'outer: for r in 0..grid.len() {
        for c in 0..grid[r].len() {
            if grid[r][c] == Cell::Unknown {
                unknown = Some((r, c));
                break 'outer;
            }
        }
    }

    let (r, c) = match unknown {
        None => return true, // fully solved
        Some(pos) => pos,
    };

    // Try Filled.
    let saved = grid.clone();
    grid[r][c] = Cell::Filled;
    if backtrack(grid, row_clues, col_clues) {
        return true;
    }

    // Try Empty.
    *grid = saved;
    grid[r][c] = Cell::Empty;
    backtrack(grid, row_clues, col_clues)
}

pub fn solve(puzzle: &Puzzle) -> Option<Vec<Vec<bool>>> {
    let mut grid = vec![vec![Cell::Unknown; puzzle.cols]; puzzle.rows];
    if backtrack(&mut grid, &puzzle.row_clues, &puzzle.col_clues) {
        Some(
            grid.iter()
                .map(|row| row.iter().map(|&c| c == Cell::Filled).collect())
                .collect(),
        )
    } else {
        None
    }
}
