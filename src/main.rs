use std::time::Instant;

use onebrc::{parse_file_path, process};

fn main() {
    let path = parse_file_path();

    // let reader = BufReader::with_capacity(BUFFER_SIZE, file);

    let start = Instant::now();

    let _results = process(path);

    println!("Took: {:?}", start.elapsed());

    let mut max_value = f64::MIN;
    let mut min_value = f64::MAX;
    let mut max_name_len = 0;

    let start = Instant::now();
    for result in &_results {
        max_value = max_value.max(result.max);
        min_value = min_value.min(result.min);
        max_name_len = max_name_len.max(result.name.len());
    }
    println!(
        "Looping through {} results took: {:?}",
        _results.len(),
        start.elapsed()
    );
    println!("max value: {max_value}\nmin value: {min_value}\nmax name length: {max_name_len}");
}
