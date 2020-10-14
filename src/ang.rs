use std::thread;
extern crate crossbeam;

const NTHREADS: usize = 4;

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


fn build_graph(dictionary: &Vec<String>) -> Vec<Vec<u8>> {
//fn build_graph() -> Vec<Vec<u8>> {
    //let s_graph = 10;
    let s_graph = dictionary.len();
    let mut graph: Vec<Vec<u8>> = vec![vec![0; s_graph]; s_graph];

    

    let mut threads = vec![];
    let mut chunk = f32::ceil(s_graph as f32 / NTHREADS as f32) as usize;
    let mut min_bound = 0;
    let mut max_bound = 0;
    //let mut base_word;

    for n_th in 0..NTHREADS {

        min_bound = n_th * chunk;
        max_bound = if min_bound + chunk > s_graph { s_graph } else { min_bound + chunk};

        threads.push(thread::spawn(move || {
            println!("thread number: {} - ini: {} - end: {}", n_th, min_bound, max_bound);

            
            for row in min_bound..max_bound {
                //base_word = &dictionary[row];
                for col in 0..s_graph {
                    /*
                    if is_one_letter_different(base_word, &dictionary[col]) {
                        graph[row][col] = 1;
                    }
                    */
                }
            }
            

        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    //build_adjacency_matrix(0, s_graph, &mut graph, &dictionary);



    graph
}


pub fn build_ladder(start: String, end: String, dictionary: Vec<String>) {

    println!("{} - {} - Number of words: {}", start, end, dictionary.len());

    let mut graph: Vec<Vec<u8>> = build_graph(&dictionary);
    //build_graph();

    /*
    let dictionary: Vec<String> = vec!["monk".to_string(), "mock".to_string(), "pock".to_string(), "pork".to_string(), "perk".to_string(), "perl".to_string()];
    let graph: Vec<Vec<u8>> = build_graph(&dictionary);
    println!("{:?}", graph);
    */
    
}

