use std::{fs::File, io::Read};

pub fn read_file() -> Vec<String> {
    let mut file = File::open("src/main.rs").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut lines = Vec::new();
    for line in contents.lines() {
        lines.push(line.to_string());
    }

    lines
}
