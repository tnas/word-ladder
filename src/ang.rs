use crossbeam_utils::thread;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::slice;

const ANG_MODE_GRAPH: &str = "-g";
const ANG_MODE_DYNAMIC: &str = "-d";
const ANG_MODE_BENCHMARK: &str = "-b";
const BENCHMARKING_SIZE: u8 = 5;


fn is_one_letter_different(baseword: &String, word: &String) -> bool {
    
    if baseword.len() != word.len() { return false; }

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
    return match dictionary.iter().position(|w| w == base_word) {
        Some(v) => v,
        None => panic!("There is no the word '{}' in the dictionary!", base_word)
    }
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



fn build_neighborhood_parallel(dictionary: &Vec<String>, start: &String, end: &String, nthreads: usize) -> (Duration, Duration, bool, Vec<usize>) {

    let time_graph = Instant::now();
    let graph: Vec<Vec<usize>> = build_graph(&dictionary, nthreads);
    let duration_graph = time_graph.elapsed();

    let time_neighborhood = Instant::now();
    let (neighborhood, found_end) = build_neighborhood(&graph, &dictionary, &start, &end);
    let duration_neighborhood = time_neighborhood.elapsed();

    let mut level = neighborhood.len();
    let mut ladder: Vec<usize> = vec![0; level];

    if found_end {

        let mut prev_index = get_word_position(&dictionary, &end);
        ladder[level - 1] = prev_index;
        level -= 1;
            
        for neighbd in (0..level).rev() {
        
            for w_index in neighborhood[neighbd].iter() {
        
                if graph[prev_index][*w_index] == 1 {
                    ladder[neighbd] = *w_index;
                    prev_index = *w_index;
                    break;
                }
            }
        }
    }
    
    return (duration_graph, duration_neighborhood, found_end, ladder);
}


fn build_ladder_parallel(dictionary: &Vec<String>, start: &String, end: &String, nthreads: usize) -> (bool, Vec<usize>) {
    
    let n_words = dictionary.len();
    let num_threads = if nthreads > n_words { n_words } else { nthreads };

    let start_index = get_word_position(&dictionary, &start);
    let word_levels: Vec<RwLock<usize>> = std::iter::repeat_with(|| RwLock::new(usize::MAX)).take(n_words).collect();
    *(word_levels[start_index].write().unwrap()) = 0;

    let is_word_processed: Vec<RwLock<bool>> = std::iter::repeat_with(|| RwLock::new(false)).take(n_words).collect();
    let level_checked_to_die: Vec<RwLock<bool>> = std::iter::repeat_with(|| RwLock::new(false)).take(n_words).collect();
    *(level_checked_to_die[0].write().unwrap()) = true;

    let arc_word_levels       = Arc::new(word_levels);
    let arc_word_processed    = Arc::new(is_word_processed);
    let arc_checked_to_die    = Arc::new(level_checked_to_die);
    let arc_found_end         = Arc::new(AtomicBool::new(false));
    let lock_level            = Arc::new(Mutex::new(0));
    let level                 = Arc::new(AtomicUsize::new(0));
    let last_processed_level  = Arc::new(AtomicUsize::new(0));
    let index_end_word        = Arc::new(AtomicUsize::new(0));
    
    thread::scope(|scope| {

        for _ in 0..num_threads {
            
            let th_word_levels          = arc_word_levels.clone();
            let th_is_word_processed    = arc_word_processed.clone();
            let th_level_checked_to_die = arc_checked_to_die.clone();
            let th_found_end            = arc_found_end.clone();
            let th_lock_level           = lock_level.clone();
            let available_level         = level.clone();
            let th_last_level           = last_processed_level.clone();
            let th_index_end_word       = index_end_word.clone();
            let mut thread_has_work     = true;

            scope.spawn(move |_| {

                while !th_found_end.load(Ordering::Relaxed) && thread_has_work { 

                    let level_lock = th_lock_level.lock().unwrap(); 
                    let th_level = available_level.load(Ordering::Relaxed);
                    available_level.fetch_add(1, Ordering::SeqCst);
                    drop(level_lock); 

                    if th_level >= n_words {
                        thread_has_work = false;
                        continue;
                    }

                    let mut last_check_before_die = false;

                    while !*th_level_checked_to_die[th_level].read().unwrap() || !last_check_before_die {

                        if *th_level_checked_to_die[th_level].read().unwrap() { last_check_before_die = true; }

                        for idbase in (0..n_words).into_iter().filter(|&i| *th_word_levels[i].read().unwrap() == th_level) {

                            if *th_is_word_processed[idbase].read().unwrap() { continue; }

                            for idcmp in (0..n_words).into_iter().filter(|&i| *th_word_levels[i].read().unwrap() > th_level) {

                                if is_one_letter_different(&dictionary[idbase], &dictionary[idcmp]) {

                                    if dictionary[idcmp] == *end {
                                        th_found_end.store(true, Ordering::Relaxed);
                                        th_last_level.store(th_level + 1, Ordering::Relaxed);
                                        th_index_end_word.store(idcmp, Ordering::Relaxed);
                                    }
                                    
                                    let mut curr_level = th_word_levels[idcmp].write().unwrap();
                                    *curr_level = th_level + 1;

                                    let mut curr_word = th_is_word_processed[idcmp].write().unwrap();
                                    *curr_word = false;
                                }
                            }

                            let mut curr_word = th_is_word_processed[idbase].write().unwrap();
                            *curr_word = true;
                        }
                    }

                    if th_level + 1 < n_words { 
                        let mut curr_level = th_level_checked_to_die[th_level + 1].write().unwrap(); 
                        *curr_level = true;
                    }
                }
            });
        }
    }).unwrap();

    let found = arc_found_end.load(Ordering::Relaxed);
    let last_level = last_processed_level.load(Ordering::Relaxed);
    let mut structure_level: Vec<usize> = vec![0; last_level + 1];

    if found {

        structure_level[last_level] = index_end_word.load(Ordering::Relaxed);
    
        for level in (0..last_level).rev() {
            for id in (0..n_words).filter(|&i| *arc_word_levels[i].read().unwrap() == level) {
                if is_one_letter_different(&dictionary[structure_level[level + 1]], &dictionary[id]) {
                    structure_level[level] = id;
                    break;
                }
            }
        }
    }

    return (found, structure_level);
}


fn print_ladder(exist: bool, start: &String, end: &String, ladder: &Vec<usize>, dictionary: &Vec<String>) {

    if exist {
        for id in 0..ladder.len() {
            print!("[{}] ", dictionary[ladder[id]]);
        }
    
        println!("\nSize of ladder: {}", ladder.len());
    }
    else {
        println!("There is no word ladder between {} and {}!", start, end);
    }
}


pub fn build_ladder(start: &String, end: &String, dictionary: &Vec<String>, mode: String, nthread: usize) {

    assert_eq!(start.len(), end.len(), "There is no word ladder between {} and {}!", start, end);

    match mode.as_str() {  

        ANG_MODE_DYNAMIC => {

            let time_ladder = Instant::now();
            let (found, ladder) = build_ladder_parallel(dictionary, start, end, nthread);
            println!("[Building ladder] CPU time: {:?}",  time_ladder.elapsed());
            print_ladder(found, start, end, &ladder, &dictionary);
        },

        ANG_MODE_GRAPH => {
            let time_ladder = Instant::now();
            let (time_graph, time_neigh, found, ladder) = build_neighborhood_parallel(dictionary, start, end, nthread);
            println!("[Building ladder total] CPU time: {:?}",  time_ladder.elapsed());
            println!("[--Building graph] CPU time: {:?}",  time_graph);
            println!("[--Building neighborhood] CPU time: {:?}",  time_neigh);
            print_ladder(found, start, end, &ladder, &dictionary);
        },

        ANG_MODE_BENCHMARK => {
            let mut total_time_dynamic = Duration::new(0, 0);
            let mut total_time_graph = Duration::new(0, 0);

            for _ in 0..BENCHMARKING_SIZE {

                let time_dynamic = Instant::now();
                build_ladder_parallel(dictionary, start, end, nthread);
                total_time_dynamic += time_dynamic.elapsed();

                let time_graph = Instant::now();
                build_neighborhood_parallel(dictionary, start, end, 1);
                total_time_graph += time_graph.elapsed();
            }

            println!("CPU time for ladder between {} and {} ...", start, end);
            println!("[Serial ANG] CPU time: {:?}",  total_time_graph.div_f32(BENCHMARKING_SIZE as f32));
            println!("[Parallel ANG] CPU time: {:?}",  total_time_dynamic.div_f32(BENCHMARKING_SIZE as f32));
        }

        &_ => panic!("Undefined option")
    }
}

