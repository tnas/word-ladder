use crossbeam_utils::thread;
use std::time::{Instant};
use std::sync::Arc;
use std::slice;

const NTHREADS: usize = 8;


fn is_one_letter_different(baseword: &String, word: &String) -> bool {
    
    let mut word_chars = word.chars();
    let mut diff_counter = 0;
    
    for letter in baseword.chars() {
        if letter != word_chars.next().unwrap() {
            diff_counter += 1;
        }
    }

    diff_counter == 1
}


fn build_graph(dictionary: Vec<String>) -> Vec<Vec<usize>> {

    let s_graph = dictionary.len();
    let mut graph: Vec<Vec<usize>> = vec![vec![0; s_graph]; s_graph];
    
    let chunk = f32::ceil(s_graph as f32 / NTHREADS as f32) as usize;
    let mut sliced_graph: Vec<&mut [Vec<usize>]> = Vec::with_capacity(NTHREADS);
    let ptr_graph = graph.as_mut_ptr();

    let mut offset;
    let mut count_of_items;

    for n_th in 0..NTHREADS {
        offset = n_th * chunk;
        count_of_items = if offset + chunk > s_graph { s_graph - offset } else { chunk };
        unsafe {
            sliced_graph.push(slice::from_raw_parts_mut(ptr_graph.offset((offset) as isize), count_of_items));
        }
    }

    thread::scope(|scope| {

        let arc_wordlist = Arc::new(dictionary);

        for (n_th, slice) in sliced_graph.iter_mut().enumerate() {
            
            let th_wordlist = Arc::clone(&arc_wordlist);

            scope.spawn(move |_| {

                let mut base_word_position;
                let shift = n_th * chunk;

                for (row_index, row) in slice.into_iter().enumerate() {

                    base_word_position = row_index + shift;

                    for (col_index, cell) in row.into_iter().enumerate() {

                        if is_one_letter_different(&th_wordlist[base_word_position], &th_wordlist[col_index]) {
                            *cell = 1;
                        }
                    }
                }
            });
        }
    }).unwrap();

    graph
}




pub fn build_ladder(start: String, end: String, dictionary: Vec<String>) {

    println!("{} - {} - Number of words: {}", start, end, dictionary.len());

    /*
    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    let _graph: Vec<Vec<usize>> = build_graph(dictionary);
    println!("{:?}", _graph);
    */
    
    let now = Instant::now();
    let _graph: Vec<Vec<usize>> = build_graph(dictionary);
    println!("Elapsed CPU time: {:?}",  now.elapsed());

    
}

