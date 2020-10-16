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



fn get_word_position(dictionary: &Vec<String>, base_word: &String) -> usize {
    return dictionary.iter().position(|w| w == base_word).unwrap();
}



fn build_graph(dictionary: &Vec<String>) -> Vec<Vec<usize>> {

    let s_graph = dictionary.len();
    let mut graph: Vec<Vec<usize>> = vec![vec![0; s_graph]; s_graph];
    
    let (num_threads, chunk) = if NTHREADS > s_graph { (s_graph, 1) } else { (NTHREADS, f32::ceil(s_graph as f32 / NTHREADS as f32) as usize) };
    let mut sliced_graph: Vec<&mut [Vec<usize>]> = Vec::with_capacity(NTHREADS);
    let ptr_graph = graph.as_mut_ptr();

    let mut offset;
    let mut count_of_items;

    for n_th in 0..num_threads {
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

                let mut word_position;
                let shift = n_th * chunk;

                for (row_index, row) in slice.into_iter().enumerate() {

                    word_position = row_index + shift;

                    for (col_index, cell) in row.into_iter().enumerate() {

                        if is_one_letter_different(&th_wordlist[word_position], &th_wordlist[col_index]) {
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


    if start.len() != end.len() {
        println!("There is no word ladder between {} and {}!", start, end);
        return;
    }

    //println!("{} - {} - Number of words: {}", start, end, dictionary.len());

    /*
    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    let graph: Vec<Vec<usize>> = build_graph(&dictionary);
    */

    let now = Instant::now();
    let graph: Vec<Vec<usize>> = build_graph(&dictionary);
    println!("[Building graph] Elapsed CPU time: {:?}",  now.elapsed());

    let mut neighborhood: Vec<Vec<usize>> = Vec::new();
    let n_words = dictionary.len();
    let mut is_word_available: Vec<bool> = vec![true; n_words];
    let mut found_end = false;

    let word_index = get_word_position(&dictionary, &start);
    neighborhood.push(vec![word_index]); 
    is_word_available[word_index] = false;
    let mut level = 0;
    let mut n_available_words = n_words;

    while !found_end && n_available_words > 0 {

        let mut next_level: Vec<usize> = Vec::new();

        'levelloop: for w_index in neighborhood[level].iter() {

            for col in 0..n_words {

                if graph[*w_index][col] == 1 && is_word_available[col] {

                    next_level.push(col);

                    if dictionary[col] == end {
                        found_end = true;
                        break 'levelloop;
                    }

                    is_word_available[col] = false;
                    n_available_words -= 1;
                }
            }
        }

        level += 1;
        neighborhood.push(next_level);
    }

    if found_end {

        let mut ladder: Vec<&String> = Vec::with_capacity(level + 1);
        ladder.push(&end);
        let mut prev_index = get_word_position(&dictionary, &end);
        
        for neighbd in (0..level).rev() {
    
            for w_index in neighborhood[neighbd].iter() {
    
                if graph[prev_index][*w_index] == 1 {
                    ladder.push(&dictionary[*w_index]);
                    prev_index = *w_index;
                    break;
                }
            }
        }
    
        while let Some(top) = ladder.pop() {
            print!("[{}] ", top);
        }
        println!();
    }
    else {
        println!("There is no word ladder between {} and {}!", start, end);
    }
}

