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
