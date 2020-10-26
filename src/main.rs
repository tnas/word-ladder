use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;

mod ang;

const LADDER_FINDER: &str = "-f";


fn read_dictionary(size_word:usize) -> Vec<String> {

    let mut word_list: Vec<String> = Vec::new();

    let file = File::open("src/dictionary.txt").unwrap();
    let reader = BufReader::new(file);

    for word in reader.lines().map(|w| w.unwrap()).filter(|w| w.len() == size_word) {
        word_list.push(word);
    }
        
    word_list
}

fn parse_arguments() -> (String, usize) {
    
    let args: Vec<String> = env::args().collect();
    let num_threads: usize;
    let mode: String;

    match args.len() {
        3 => {
            mode = String::from(&args[1]);
            num_threads = match args[2].parse::<usize>() {
                Ok(n) => n,
                Err(_) => panic!("error: second argument not an integer")
            };
        }
        _ => panic!("Pass -g for graph builder or -d for dynamic neighborhood and the number of threads!")
        
    }

    return (mode, num_threads);
}


fn main() {

    let contents = fs::read_to_string("src/input.txt")
        .expect("Wrong file name!");
    let (ang_mode, nthreads) = parse_arguments();

    if ang_mode == LADDER_FINDER {
        
        let word_list = read_dictionary(nthreads);
        let wsize = word_list.len();
        
        for base in 0..wsize - 1 {
            for cmp in base + 1 .. wsize {
                ang::build_ladder(&String::from(&word_list[base]), &String::from(&word_list[cmp]), &word_list, "-d".to_string(), 4);
            }
        }
        
    }
    else {
        for line in contents.lines() {

            let tokens: Vec<_> = line.split_whitespace().collect();
    
            let word_list = read_dictionary(tokens[0].len());
    
            println!("Searching ladder between {} and {} ...", tokens[0].to_string(), tokens[1].to_string());
            ang::build_ladder(&tokens[0].to_string(), &tokens[1].to_string(), &word_list, ang_mode.to_string(), nthreads);     
        }
    }
}
