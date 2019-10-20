use std::fs::File;
use std::io::{BufReader, BufRead};

/// open the given filename and return a Vec of lines
pub fn editor_open(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let f = File::open(filename)?;
    let reader = BufReader::new(f);

    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

