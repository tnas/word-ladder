use crossbeam_utils::thread;
use std::time::{Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::slice;

const ANG_MODE_GRAPH: &str = "-g";
const ANG_MODE_DYNAMIC: &str = "-d";


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



fn build_graph(dictionary: &Vec<String>, nthreads: usize) -> Vec<Vec<usize>> {

    let s_graph = dictionary.len();
    let mut graph: Vec<Vec<usize>> = vec![vec![0; s_graph]; s_graph];
    
    let (num_threads, chunk) = if nthreads > s_graph { (s_graph, 1) } else { (nthreads, f32::ceil(s_graph as f32 / nthreads as f32) as usize) };
    let mut sliced_graph: Vec<&mut [Vec<usize>]> = Vec::with_capacity(nthreads);
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



fn build_neighborhood_parallel(dictionary: &Vec<String>, start: &String, end: &String, nthreads: usize) {

    let time_graph = Instant::now();
    let graph: Vec<Vec<usize>> = build_graph(&dictionary, nthreads);
    println!("[Building graph] CPU time: {:?}",  time_graph.elapsed());

    let time_neighborhood = Instant::now();
    let (neighborhood, found_end) = build_neighborhood(&graph, &dictionary, &start, &end);
    println!("[Building neighborhood] CPU time: {:?}",  time_neighborhood.elapsed());

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
}


fn build_ladder_parallel(dictionary: &Vec<String>, start: &String, end: &String, nthreads: usize) -> Vec<usize> {
    
    let n_words = dictionary.len();
    let num_threads = if nthreads > n_words { n_words } else { nthreads };

    let start_index = get_word_position(&dictionary, &start);
    let word_levels: Vec<RwLock<usize>> = std::iter::repeat_with(|| RwLock::new(usize::MAX)).take(n_words).collect();
    *(word_levels[start_index].write().unwrap()) = 0;

    let thread_levels: Vec<RwLock<usize>> = std::iter::repeat_with(|| RwLock::new(usize::MAX)).take(num_threads).collect();

    let arc_word_levels       = Arc::new(word_levels);
    let arc_thread_levels     = Arc::new(thread_levels);
    let arc_found_end         = Arc::new(AtomicBool::new(false));
    let lock_level            = Arc::new(Mutex::new(0));
    let level                 = Arc::new(AtomicUsize::new(0));
    let last_processed_level  = Arc::new(AtomicUsize::new(0));
    let index_end_word        = Arc::new(AtomicUsize::new(0));
    
    thread::scope(|scope| {

        for n_th in 0..num_threads {
            
            let th_word_levels       = arc_word_levels.clone();
            let thread_level_alive   = arc_thread_levels.clone();
            let th_found_end         = arc_found_end.clone();
            let th_lock_level        = lock_level.clone();
            let available_level      = level.clone();
            let th_last_level        = last_processed_level.clone();
            let th_index_end_word    = index_end_word.clone();

            scope.spawn(move |_| {

                while !th_found_end.load(Ordering::Relaxed) { 

                    let level_lock = th_lock_level.lock().unwrap(); 
                    let th_level = available_level.load(Ordering::Relaxed);
                    available_level.fetch_add(1, Ordering::SeqCst);
                    *thread_level_alive[n_th].write().unwrap() = th_level;
                    drop(level_lock); 

                    let mut keep_alive = true;

                    while keep_alive {

                        for idbase in (0..n_words).into_iter().filter(|&i| *th_word_levels[i].read().unwrap() == th_level) {
                            
                            for idcmp in (0..n_words).into_iter().filter(|&i| *th_word_levels[i].read().unwrap() > th_level) {

                                if is_one_letter_different(&dictionary[idbase], &dictionary[idcmp]) {

                                    if dictionary[idcmp] == *end {
                                        th_found_end.store(true, Ordering::Relaxed);
                                        th_last_level.store(th_level + 1, Ordering::Relaxed);
                                        th_index_end_word.store(idcmp, Ordering::Relaxed);
                                    }

                                    *th_word_levels[idcmp].write().unwrap() = th_level + 1;
                                }
                            }
                        }

                        // Checking if thread must keep alive
                        keep_alive = false;
                        if th_level > 0 {
                            for id in 0..num_threads {
                                if *thread_level_alive[id].read().unwrap() == th_level - 1 {
                                    keep_alive = true;
                                    break;
                                }
                            }
                        }

                        if !keep_alive { 
                            *thread_level_alive[n_th].write().unwrap() = usize::MAX;
                        }
                    }  
                }
            });
        }
    }).unwrap();


    let last_level = last_processed_level.load(Ordering::Relaxed);
    let mut structure_level: Vec<usize> = vec![0; last_level + 1];
    structure_level[last_level] = index_end_word.load(Ordering::Relaxed);

    for level in (0..last_level).rev() {
        for id in (0..n_words).filter(|&i| *arc_word_levels[i].read().unwrap() == level) {
            if is_one_letter_different(&dictionary[structure_level[level + 1]], &dictionary[id]) {
                structure_level[level] = id;
                break;
            }
        }
    }

    return structure_level;
}



pub fn build_ladder(start: String, end: String, dictionary: Vec<String>, mode: String, nthread: usize) {

    if start.len() != end.len() {
        println!("There is no word ladder between {} and {}!", start, end);
        return;
    }
    /*
    println!("{} - {} - Number of words: {}", start, end, dictionary.len());
    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    let graph: Vec<Vec<usize>> = build_graph(&dictionary);
    */

    match mode.as_str() {        
        ANG_MODE_DYNAMIC => {
            let time_ladder = Instant::now();
            let ladder: Vec<usize> = build_ladder_parallel(&dictionary, &start, &end, nthread);
            println!("[Building ladder] CPU time: {:?}",  time_ladder.elapsed());
        
            for id in 0..ladder.len() {
                print!("[{}] ", dictionary[ladder[id]]);
            }
        
            println!("\nSize of ladder: {}", ladder.len());
        },
        ANG_MODE_GRAPH => build_neighborhood_parallel(&dictionary, &start, &end, nthread),
        &_ => panic!("Undefined option")
    }
}

