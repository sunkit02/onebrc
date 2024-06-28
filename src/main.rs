use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

const BUFFER_SIZE: usize = 3 * 1024;

fn main() {
    let path = parse_file_path();

    let file = File::open(path).unwrap();
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);

    let start = Instant::now();

    let _results = parse_lines(reader);

    println!("Took: {:?}", start.elapsed());
}

struct StationAggregateTmp {
    min: f64,
    max: f64,
    total: f64,
    count: u64,
}

#[derive(Debug, PartialEq)]
struct StationAggregate {
    name: String,
    min: f64,
    max: f64,
    mean: f64,
}

fn parse_lines<R: Read>(reader: BufReader<R>) -> Vec<StationAggregate> {
    let mut results = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let (name, temp) = line
            .split_once(';')
            .map(|(name, temp)| (name.to_owned(), temp.parse().unwrap()))
            .unwrap();

        let entry = results.entry(name).or_insert(StationAggregateTmp {
            min: temp,
            max: temp,
            total: 0f64,
            count: 0,
        });

        if temp < entry.min {
            entry.min = temp;
        } else if temp > entry.max {
            entry.max = temp;
        }
        entry.total += temp;
        entry.count += 1;
    }

    let mut results = results
        .into_iter()
        .map(|(name, aggregate)| StationAggregate {
            name,
            min: aggregate.min,
            max: aggregate.max,
            mean: aggregate.total / aggregate.count as f64,
        })
        .collect::<Vec<_>>();

    results.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    results
}

const USAGE_MSG: &str = "Usage: onebrc <data file path>";

fn parse_file_path() -> PathBuf {
    let args = std::env::args().collect::<Vec<String>>();
    let path_str = match args.len() {
        1 => {
            eprintln!("{}", USAGE_MSG);
            std::process::exit(1);
        }
        2 => &args[1],
        _ => {
            eprintln!("{}", USAGE_MSG);
            std::process::exit(1);
        }
    };

    PathBuf::from_str(path_str).expect("Failed to parse file path.")
}

fn count_lines_read_to_buf(path: PathBuf) {
    let mut f = File::open(path).unwrap();
    let mut buf = [0u8; 1024 * 3];
    let mut lines = 0;
    loop {
        let read = f.read(&mut buf).unwrap();
        if read == 0 {
            break;
        }
        lines += buf[..read].iter().filter(|&&byte| byte == 0xA).count();
    }

    println!("{lines}");
}

fn count_lines_read_line(path: PathBuf) {
    let f = File::open(path).unwrap();
    let reader = BufReader::with_capacity(3 * 1024, f);
    let lines = reader.lines().count();

    println!("{lines}");
}

fn count_lines_concurrent(path: PathBuf, threads: usize) {
    let mut file = File::open(&path).unwrap();
    let file_size = file.metadata().unwrap().size() as usize;
    let section_size = file_size / threads;
    let remaining = file_size % threads;

    let mut handles = Vec::with_capacity(threads);
    for i in 0..threads {
        let path_cloned = path.clone();
        let start = i * section_size;
        let handle = std::thread::spawn(move || {
            let mut file = File::open(path_cloned).unwrap();
            file.seek(std::io::SeekFrom::Start(start as u64)).unwrap();
            let mut buf = [0u8; 3 * 1024];
            let mut read = 0;
            let mut lines = 0;
            loop {
                let mut bytes = file.read(&mut buf).unwrap();
                if read + bytes > section_size {
                    bytes = section_size - read;
                }

                lines += buf[..bytes].iter().filter(|&&byte| byte == 0xA).count();

                read += bytes;
                if read >= section_size {
                    break;
                }
            }

            println!(
                "Spawning thread to read {} to {}, lines read -> {lines}",
                start,
                start + section_size
            );

            lines as u64
        });
        handles.push(handle);
    }

    let remaining = if remaining > 0 {
        println!("Reading remaining {remaining} bytes.");

        file.seek(std::io::SeekFrom::Start((section_size * threads) as u64))
            .unwrap();
        let mut buf = Vec::with_capacity(remaining);
        let read = file.read_to_end(&mut buf).unwrap();
        buf[..read].iter().filter(|&&byte| byte == 0xA).count() as u64
    } else {
        0
    };

    let lines = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .sum::<u64>();

    println!(
        "lines {lines} + remaining {remaining} = {}",
        lines + remaining
    );
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    use super::*;

    #[test]
    // Just a little test to add some confidence to the results' correctness
    fn can_parse_lines() {
        let input = r#"station1;99.9
station3;23.0
station1;-67.4
station1;-55.8
station2;43.3
station2;81.8
station3;-82.2
station2;10.1
station3;-99.9
"#;

        let reader = BufReader::new(Cursor::new(input));
        let mut results = parse_lines(reader);

        // Round the means to side-step the floating point number imprecision issue
        results
            .iter_mut()
            .for_each(|result| result.mean = result.mean.round());

        let expected = [
            StationAggregate {
                name: "station1".to_owned(),
                min: -67.4,
                max: 99.9,
                mean: -8.0,
            },
            StationAggregate {
                name: "station2".to_owned(),
                min: 10.1,
                max: 81.8,
                mean: 45.0,
            },
            StationAggregate {
                name: "station3".to_owned(),
                min: -99.9,
                max: 23.0,
                mean: -53.0,
            },
        ];

        assert_eq!(results, expected);
    }
}
