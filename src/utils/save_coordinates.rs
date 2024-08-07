use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn save_colors_pos(filename: &str, colors_pos: &Vec<(f64, f64)>) -> io::Result<()> {
    let mut file = File::create(filename)?;
    for &(x, y) in colors_pos.iter() {
        writeln!(file, "{} {}", x, y)?;
    }
    Ok(())
}

pub fn load_colors_pos(filename: &str) -> io::Result<Vec<(f64, f64)>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut colors_pos = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let coords: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse::<f64>().expect("Invalid coordinate value"))
            .collect();
        if coords.len() == 2 {
            colors_pos.push((coords[0], coords[1]));
        }
    }
    Ok(colors_pos)
}
