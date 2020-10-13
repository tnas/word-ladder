use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod ang;


fn read_dictionary(size_word:usize) -> Vec<String> {

    let mut word_list: Vec<String> = Vec::new();

    let file = File::open("dictionary.txt").unwrap();
    let reader = BufReader::new(file);

    for word in reader.lines().map(|w| w.unwrap()).filter(|w| w.len() == size_word) {
        word_list.push(word);
    }
        
    word_list
}


fn main() {

    let contents = fs::read_to_string("input.txt")
        .expect("Wrong file name!");

    for line in contents.split("\n") {

        let tokens: Vec<&str> = line.split_whitespace().collect();

        let word_list = read_dictionary(tokens[0].to_string().len());

        ang::build_ladder(tokens[0].to_string(), tokens[1].to_string());

        println!("Number of words: {}", word_list.len());        
    }
}
