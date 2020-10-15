use crossbeam_utils::thread;
use std::time::Duration;
use cpu_time::ProcessTime;
use std::sync::Arc;
use std::slice;

const NTHREADS: usize = 2;

/*
fn build_adjacency_matrix(ini: usize, end: usize, matrix: &mut Vec<Vec<u8>>, dictionary: &Vec<String>) {

    let nwords = dictionary.len();
    let mut base_word;

    for row in ini..end {
        base_word = &dictionary[row];
        for col in 0..nwords {
            if is_one_letter_different(base_word, &dictionary[col]) {
                matrix[row][col] = 1;
            }
        }
    }
}
*/

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

    println!("chunk: {}", chunk);

    let mut offset;
    let mut count_of_items;

    for n_th in 0..NTHREADS {
        offset = n_th * chunk;
        count_of_items = if offset + chunk > s_graph { s_graph - offset } else { chunk };
        println!("thread: {} - offset: {} - counter: {}", n_th, offset, count_of_items);
        unsafe {
            sliced_graph.push(slice::from_raw_parts_mut(ptr_graph.offset((offset) as isize), count_of_items));
        }
    }

    thread::scope(|scope| {

        let arc_wordlist = Arc::new(dictionary);
        let mut n_th: usize = 0;
        let mut min_bound;
        let mut max_bound;

        for slice in &mut sliced_graph {
            
            min_bound = n_th * chunk;
            max_bound = if min_bound + chunk > s_graph { s_graph } else { min_bound + chunk};

            let th_wordlist = Arc::clone(&arc_wordlist);

            scope.spawn(move |_| {

                println!("thread number: {} - ini: {} - end: {}", n_th, min_bound, max_bound);

                for row in min_bound..max_bound {

                    for col in 0..s_graph {

                        if is_one_letter_different(&th_wordlist[row], &th_wordlist[col]) {
                            println!("th:{} -->> set row:{} col:{}", n_th, row, col);
                            (*slice)[row][col] = 1;
                        }
                    }
                }

                println!("thread number: {} - Finished!", n_th);
            });

            n_th += 1;
        }
    }).unwrap();

    graph
}


pub fn build_ladder(start: String, end: String, dictionary: Vec<String>) {

    println!("{} - {} - Number of words: {}", start, end, dictionary.len());

    
    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    let _graph: Vec<Vec<usize>> = build_graph(dictionary);
    println!("{:?}", _graph);
    
/*

    let start = ProcessTime::try_now().expect("Getting process time failed.");
    let _graph: Vec<Vec<usize>> = build_graph(dictionary);
    let cpu_time: Duration = start.try_elapsed().expect("Getting process time failed.");
    println!("Elapsed CPU time: {:?}", cpu_time);

*/
    
}

