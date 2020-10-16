use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod ang;


fn read_dictionary(size_word:usize) -> Vec<String> {

    let mut word_list: Vec<String> = Vec::new();

    let file = File::open("src/dictionary.txt").unwrap();
    let reader = BufReader::new(file);

    for word in reader.lines().map(|w| w.unwrap()).filter(|w| w.len() == size_word) {
        word_list.push(word);
    }
        
    word_list
}


fn main() {

    let contents = fs::read_to_string("src/input.txt")
        .expect("Wrong file name!");

    for line in contents.lines() {

        let tokens: Vec<_> = line.split_whitespace().collect();

        let word_list = read_dictionary(tokens[0].len());

        println!("Searching ladder between {} and {} ...", tokens[0].to_string(), tokens[1].to_string());
        ang::build_ladder(tokens[0].to_string(), tokens[1].to_string(), word_list);     
    }
}
