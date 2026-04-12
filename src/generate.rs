use rand::Rng;

use crate::puzzle::Puzzle;

pub fn generate(size: usize) -> Puzzle {
    let size = size.max(1);
    let mut rng = rand::thread_rng();

    loop {
        let grid: Vec<Vec<bool>> = (0..size)
            .map(|_| (0..size).map(|_| rng.gen::<bool>()).collect())
            .collect();

        // Reject all-empty grids (unplayable).
        if grid.iter().any(|row| row.iter().any(|&b| b)) {
            return Puzzle::new(format!("Random {}×{}", size, size), grid);
        }
    }
}
