use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn progress_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".picrossh_progress"))
}

pub fn load_best_times() -> HashMap<String, u64> {
    let mut map = HashMap::new();
    if let Some(path) = progress_path() {
        if let Ok(contents) = fs::read_to_string(&path) {
            for line in contents.lines() {
                let mut parts = line.splitn(2, '\t');
                if let (Some(name), Some(ms_str)) = (parts.next(), parts.next()) {
                    if let Ok(ms) = ms_str.parse::<u64>() {
                        map.insert(name.to_string(), ms);
                    }
                }
            }
        }
    }
    map
}

pub fn save_best_times(times: &HashMap<String, u64>) {
    if let Some(path) = progress_path() {
        let mut content = String::new();
        let mut pairs: Vec<_> = times.iter().collect();
        pairs.sort_by_key(|(k, _)| k.as_str());
        for (name, ms) in pairs {
            content.push_str(&format!("{}\t{}\n", name, ms));
        }
        let _ = fs::write(&path, content);
    }
}
