use std::env;
use colored::*;
use std::fs::File;
use std::io::prelude::*;

// usage : regex string pattern
// currently not support standard input

mod nfa;
mod re;

use nfa::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let re_match_engine = Nfa::new(&args[1]);

    let mut file = File::open(&args[2])
                        .expect("Failed to load the file");
    
    let mut content = String::new();
    file.read_to_string(&mut content)
                        .expect("Failed to read the content");

    let mut range = re_match_engine.partial_match(&content);
    
    range.sort_unstable_by(|a, b| {
        if a.0 != b.0 {
            a.0.cmp(&b.0)
        } else {
            b.1.cmp(&a.1)
        }
    });
    range.dedup_by(|a, b| {
        a.0 == b.0
    });


    let mut last = 0;
    for (l, r) in range { 
        if l < last {
            continue;
        }
        print!("{}{}", &content[last..l], &content[l..=r].red());
        last = r + 1;
    }
    println!("{}", &content[last..]);
}
   
