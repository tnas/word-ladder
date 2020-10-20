use crossbeam_utils::thread;
use std::time::{Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
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


#[inline]
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


fn build_neighborhood(graph: &Vec<Vec<usize>>, dictionary: &Vec<String>, start: &String, end: &String) -> (Vec<Vec<usize>>, bool) {

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

                    if dictionary[col] == *end {
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

    return (neighborhood, found_end);
}



fn build_neighborhood_parallel(dictionary: &Vec<String>, start: &String, end: &String) -> Vec<usize> {
    
    let n_words = dictionary.len();
    let num_threads = if NTHREADS > n_words { n_words } else { NTHREADS };
    //let num_threads = 1;

    let start_index = get_word_position(&dictionary, &start);
    let word_levels: Vec<RwLock<usize>> = std::iter::repeat_with(|| RwLock::new(usize::MAX)).take(n_words).collect();
    *(word_levels[start_index].write().unwrap()) = 0;

    let thread_levels: Vec<RwLock<usize>> = std::iter::repeat_with(|| RwLock::new(usize::MAX)).take(num_threads).collect();

    let arc_word_levels = Arc::new(word_levels);
    let arc_thread_levels = Arc::new(thread_levels);
    let arc_n_available_words = Arc::new(AtomicUsize::new(n_words - 1));
    let arc_found_end = Arc::new(AtomicBool::new(false));
    let lock_level = Arc::new(Mutex::new(0));
    let level = Arc::new(AtomicUsize::new(0));
    
    thread::scope(|scope| {

        for n_th in 0..num_threads {
            
            let th_word_levels = Arc::clone(&arc_word_levels);
            let thread_level_alive = Arc::clone(&arc_thread_levels);
            let th_n_available_words = Arc::clone(&arc_n_available_words);
            let th_found_end = Arc::clone(&arc_found_end);
            let th_lock_level = Arc::clone(&lock_level);
            let available_level = Arc::clone(&level);

            scope.spawn(move |_| {

                while !th_found_end.load(Ordering::Relaxed) && th_n_available_words.load(Ordering::Relaxed) > 0 {

                    let level_lock = th_lock_level.lock().unwrap(); 
                    let th_level = available_level.load(Ordering::Relaxed);
                    available_level.fetch_add(1, Ordering::SeqCst);
                    *thread_level_alive[n_th].write().unwrap() = th_level;
                    println!("thread {} working on level {}", n_th, th_level);
                    drop(level_lock); 

                    loop {

                        for idbase in (0..n_words).into_iter().filter(|&i| *th_word_levels[i].read().unwrap() == th_level) {

                            for idcmp in (0..n_words).into_iter().filter(|&i| *th_word_levels[i].read().unwrap() == usize::MAX) {

                                if is_one_letter_different(&dictionary[idbase], &dictionary[idcmp]) {

                                    if dictionary[idcmp] == *end {
                                        th_found_end.store(true, Ordering::Relaxed);
                                        println!("thread {} - End element has been found at position {} - pair ({}, {})", n_th, idcmp, &dictionary[idbase], &dictionary[idcmp]);
                                    }

                                    *th_word_levels[idcmp].write().unwrap() = th_level + 1;
                                    th_n_available_words.fetch_sub(1, Ordering::SeqCst);
                                    println!("thread {} set level {} for position {} - pair ({}, {})", n_th, th_level + 1, idcmp, &dictionary[idbase], &dictionary[idcmp]);
                                }
                            }
                        }

                        let keep_alive = th_level > 0 && (0..num_threads).into_iter().any(|id| *thread_level_alive[id].read().unwrap() == th_level - 1);
                        if !keep_alive { break; }
                    }  

                   println!("thread {} testing stop condition: {} && {}", n_th, !th_found_end.load(Ordering::Relaxed), th_n_available_words.load(Ordering::Relaxed) > 0);
                }

                println!("thread {} finished!", n_th);
            });
        }
    }).unwrap();

    println!("available words after: {}", arc_n_available_words.load(Ordering::Relaxed));

    let mut structure_level = vec![usize::MAX; n_words];
    for id in 0..n_words {
        structure_level[id] = *arc_word_levels[id].read().unwrap();
    }

    println!("{:?}", structure_level);

    return structure_level;
}



pub fn build_ladder(start: String, end: String, dictionary: Vec<String>) {

    if start.len() != end.len() {
        println!("There is no word ladder between {} and {}!", start, end);
        return;
    }

    /*
    println!("{} - {} - Number of words: {}", start, end, dictionary.len());
    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    let graph: Vec<Vec<usize>> = build_graph(&dictionary);
    */

    /*
    let time_graph = Instant::now();
    let graph: Vec<Vec<usize>> = build_graph(&dictionary);
    println!("[Building graph] CPU time: {:?}",  time_graph.elapsed());

    let time_neighborhood = Instant::now();
    let (neighborhood, found_end) = build_neighborhood(&graph, &dictionary, &start, &end);
    println!("[Building neighborhood] CPU time: {:?}",  time_neighborhood.elapsed());
    */

    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    build_neighborhood_parallel(&dictionary, &start, &end);

    /*
    if found_end {

        let level = neighborhood.len();
        let mut ladder: Vec<&String> = Vec::with_capacity(level);
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
    */
}

