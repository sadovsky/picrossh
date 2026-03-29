#![allow(dead_code)]
#[path = "../puzzle.rs"]
mod puzzle;

use std::fs;
use std::path::Path;

fn main() {
    let out_dir = Path::new("puzzles");
    fs::create_dir_all(out_dir).expect("failed to create puzzles/ directory");

    let presets = puzzle::Puzzle::presets();
    println!("Exporting {} puzzles to NON format...", presets.len());

    for p in &presets {
        let filename = p.name.to_lowercase().replace(' ', "_") + ".non";
        let path = out_dir.join(&filename);

        let mut content = String::new();
        content.push_str(&format!("title {}\n", p.name));
        content.push_str(&format!("width {}\n", p.cols));
        content.push_str(&format!("height {}\n", p.rows));
        content.push_str("rows\n");
        for row_clue in &p.row_clues {
            let s = row_clue.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
            content.push_str(&s);
            content.push('\n');
        }
        content.push_str("columns\n");
        for col_clue in &p.col_clues {
            let s = col_clue.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
            content.push_str(&s);
            content.push('\n');
        }

        fs::write(&path, &content).expect("failed to write NON file");
        println!("  wrote {}", path.display());
    }
    println!("Done.");
}
