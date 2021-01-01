pub mod grid;
pub mod nom;

use std::fs;
pub fn get_data_from_file(name: &str) -> Option<String> {
    let path = format!("data/{}.txt", name);

    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) => {
            println!("Error reading data: {}", e);
            None
        }
    }
}

pub fn get_data_from_file_res(name: &str) -> std::io::Result<String> {
    let path = format!("data/{}.txt", name);
    fs::read_to_string(path)
}

pub fn lines_to_longs(contents: &str) -> Vec<i64> {
    let mut ints = Vec::new();
    for s in contents.split_ascii_whitespace() {
        ints.push(s.parse::<i64>().unwrap());
    }
    ints
}

pub fn ints_to_longs(ints: &[i32]) -> Vec<i64> {
    let longs: Vec<i64>;
    longs = ints.iter().map(|&x| x as i64).collect();
    longs
}

pub fn csv_string_to_ints(contents: &str) -> Vec<i32> {
    let mut ints = Vec::new();
    for s in contents.split(',') {
        ints.push(s.trim().parse::<i32>().unwrap());
    }
    ints
}
