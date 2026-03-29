#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Empty,
    Filled,
    Crossed,
}

pub struct Puzzle {
    pub name: &'static str,
    pub rows: usize,
    pub cols: usize,
    pub solution: Vec<Vec<bool>>,
    pub row_clues: Vec<Vec<usize>>,
    pub col_clues: Vec<Vec<usize>>,
}

fn derive_clues(line: &[bool]) -> Vec<usize> {
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

// ── Logic solver ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum LineCell {
    Unknown,
    Filled,
    Empty,
}

#[allow(dead_code)]
/// Find the leftmost valid block placement for `clues` given `known` constraints.
/// Uses recursive search to correctly handle known-Filled cells that force block positions.
fn leftmost_placement(clues: &[usize], known: &[LineCell]) -> Option<Vec<usize>> {
    fn place(
        clues: &[usize],
        known: &[LineCell],
        block: usize,
        pos: usize,
        out: &mut Vec<usize>,
    ) -> bool {
        let n = known.len();
        if block == clues.len() {
            // No remaining blocks: ensure no uncovered Filled cells remain
            return !(pos..n).any(|p| known[p] == LineCell::Filled);
        }
        let clue = clues[block];
        let mut start = pos;
        loop {
            if start + clue > n {
                return false;
            }
            // Skip known-Empty at start
            if known[start] == LineCell::Empty {
                // Can't skip a Filled cell — but start is Empty so just advance
                start += 1;
                continue;
            }
            // Check block range for known-Empty
            let bad = (start..start + clue).find(|&p| known[p] == LineCell::Empty);
            if let Some(bad_pos) = bad {
                // Any Filled cells in [start, bad_pos) would be abandoned
                if (start..bad_pos).any(|p| known[p] == LineCell::Filled) {
                    return false;
                }
                start = bad_pos + 1;
                continue;
            }
            // Check that the cell immediately after the block is not Filled (would extend it)
            if start + clue < n && known[start + clue] == LineCell::Filled {
                // Can't advance if current start is a Filled cell
                if known[start] == LineCell::Filled {
                    return false;
                }
                start += 1;
                continue;
            }
            // Try placing block here
            out.push(start);
            if place(clues, known, block + 1, start + clue + 1, out) {
                return true;
            }
            out.pop();
            // Can't advance if current start is Filled
            if known[start] == LineCell::Filled {
                return false;
            }
            start += 1;
        }
    }

    let mut positions = Vec::with_capacity(clues.len());
    if place(clues, known, 0, 0, &mut positions) {
        Some(positions)
    } else {
        None
    }
}

#[allow(dead_code)]
/// Find the rightmost valid block placement (mirror of leftmost).
fn rightmost_placement(clues: &[usize], known: &[LineCell]) -> Option<Vec<usize>> {
    let n = known.len();
    // Reverse both clues and known, find leftmost, then un-reverse positions
    let rev_known: Vec<LineCell> = known.iter().rev().copied().collect();
    let rev_clues: Vec<usize> = clues.iter().rev().copied().collect();
    let rev_positions = leftmost_placement(&rev_clues, &rev_known)?;
    // Convert back: position in reversed line → position in original line
    let k = clues.len();
    let mut positions = vec![0usize; k];
    for (i, &rp) in rev_positions.iter().enumerate() {
        let original_block = k - 1 - i;
        let original_start = n - rp - rev_clues[i];
        positions[original_block] = original_start;
    }
    Some(positions)
}

#[allow(dead_code)]
/// Deduce cells in a single line using overlap of leftmost/rightmost placements.
fn solve_line(clues: &[usize], known: &[LineCell]) -> Option<Vec<LineCell>> {
    let n = known.len();
    // Special case: all-zero clue means all cells are empty
    if clues == [0] {
        return Some(vec![LineCell::Empty; n]);
    }
    let left = leftmost_placement(clues, known)?;
    let right = rightmost_placement(clues, known)?;

    let mut result = known.to_vec();

    for (i, &clue) in clues.iter().enumerate() {
        let l = left[i];
        let r = right[i];
        // Overlap region: cells that are Filled in BOTH leftmost and rightmost placements
        if r < l + clue {
            for p in r..l + clue {
                if result[p] == LineCell::Unknown {
                    result[p] = LineCell::Filled;
                }
            }
        }
    }

    // Cells not reachable by any block in any valid placement → Empty
    // A cell p is reachable by block i if p in [left[i], right[i]+clue)
    let mut reachable = vec![false; n];
    for (i, &clue) in clues.iter().enumerate() {
        for p in left[i]..right[i] + clue {
            if p < n {
                reachable[p] = true;
            }
        }
    }
    for p in 0..n {
        if !reachable[p] && result[p] == LineCell::Unknown {
            result[p] = LineCell::Empty;
        }
    }

    Some(result)
}

impl Puzzle {
    pub fn new(name: &'static str, solution: Vec<Vec<bool>>) -> Self {
        let rows = solution.len();
        let cols = if rows > 0 { solution[0].len() } else { 0 };

        let row_clues: Vec<Vec<usize>> = solution.iter().map(|row| derive_clues(row)).collect();

        let col_clues: Vec<Vec<usize>> = (0..cols)
            .map(|c| {
                let col: Vec<bool> = solution.iter().map(|row| row[c]).collect();
                derive_clues(&col)
            })
            .collect();

        Puzzle { name, rows, cols, solution, row_clues, col_clues }
    }

    /// Returns true if this puzzle can be solved by logic alone (no guessing required).
    #[allow(dead_code)]
    pub fn is_uniquely_solvable(&self) -> bool {
        let mut grid = vec![vec![LineCell::Unknown; self.cols]; self.rows];
        loop {
            let mut changed = false;
            for r in 0..self.rows {
                let known: Vec<LineCell> = grid[r].clone();
                if let Some(deduced) = solve_line(&self.row_clues[r], &known) {
                    for c in 0..self.cols {
                        if grid[r][c] == LineCell::Unknown && deduced[c] != LineCell::Unknown {
                            grid[r][c] = deduced[c];
                            changed = true;
                        }
                    }
                }
            }
            for c in 0..self.cols {
                let known: Vec<LineCell> = (0..self.rows).map(|r| grid[r][c]).collect();
                if let Some(deduced) = solve_line(&self.col_clues[c], &known) {
                    for r in 0..self.rows {
                        if grid[r][c] == LineCell::Unknown && deduced[r] != LineCell::Unknown {
                            grid[r][c] = deduced[r];
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        grid.iter().all(|row| row.iter().all(|&c| c != LineCell::Unknown))
    }

    pub fn presets() -> Vec<Puzzle> {
        let f = false;
        let t = true;
        vec![
            // ── 5×5 ──────────────────────────────────────────────────────────
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
            Puzzle::new(
                "Hourglass",
                vec![
                    vec![t, t, t, t, t],
                    vec![f, t, f, t, f],
                    vec![f, f, t, f, f],
                    vec![f, t, f, t, f],
                    vec![t, t, t, t, t],
                ],
            ),
            Puzzle::new(
                "Diamond",
                vec![
                    vec![f, f, t, f, f],
                    vec![f, t, t, t, f],
                    vec![t, t, t, t, t],
                    vec![f, t, t, t, f],
                    vec![f, f, t, f, f],
                ],
            ),
            Puzzle::new(
                "Steps",
                vec![
                    vec![t, f, f, f, f],
                    vec![t, t, f, f, f],
                    vec![t, t, t, f, f],
                    vec![t, t, t, t, f],
                    vec![t, t, t, t, t],
                ],
            ),
            Puzzle::new(
                "Z Shape",
                vec![
                    vec![t, t, t, t, t],
                    vec![f, f, f, t, f],
                    vec![f, f, t, f, f],
                    vec![f, t, f, f, f],
                    vec![t, t, t, t, t],
                ],
            ),
            // ── 10×10 ────────────────────────────────────────────────────────
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
            Puzzle::new(
                "Letter T",
                vec![
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                    vec![f, f, f, f, t, t, f, f, f, f],
                ],
            ),
            Puzzle::new(
                "Ladder",
                vec![
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                    vec![t, t, t, t, t, t, t, t, t, t],
                    vec![t, t, f, f, f, f, f, f, t, t],
                ],
            ),
            // ── 15×15 ────────────────────────────────────────────────────────
            Puzzle::new(
                "Frame",
                vec![
                    vec![t, t, t, t, t, t, t, t, t, t, t, t, t, t, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, t, t, t, t, t, t, t, t, t, t, t, t, t, t],
                ],
            ),
            Puzzle::new(
                "Big Cross",
                vec![
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![t, t, t, t, t, t, t, t, t, t, t, t, t, t, t],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                    vec![f, f, f, f, f, f, f, t, f, f, f, f, f, f, f],
                ],
            ),
            Puzzle::new(
                "H Letter",
                vec![
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, t, t, t, t, t, t, t, t, t, t, t, t, t, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                    vec![t, f, f, f, f, f, f, f, f, f, f, f, f, f, t],
                ],
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_uniquely_solvable() {
        for p in Puzzle::presets() {
            let ok = p.is_uniquely_solvable();
            println!("{}: {}", p.name, if ok { "OK" } else { "FAIL" });
            assert!(ok, "Puzzle '{}' is not uniquely solvable without guessing", p.name);
        }
    }
}
