use core::f64;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    str::FromStr,
};

pub const MAX_NAME_LEN: usize = 100;
pub const MAX_TEMP_LEN: usize = "-99.9".len();
pub const MAX_LINE_LEN: usize = MAX_NAME_LEN + ";".len() + MAX_TEMP_LEN + "\n".len();

pub const MAX_TEMP_VALUE: f64 = 99.9;
pub const MIN_TEMP_VALUE: f64 = -99.9;

pub const BUFFER_SIZE: usize = 1024 * 1024;

pub struct StationAggregateTmp {
    min: f64,
    max: f64,
    total: f64,
    count: u64,
}

impl Default for StationAggregateTmp {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            total: 0f64,
            count: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StationAggregate {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

pub fn parse_lines<R: Read>(mut reader: BufReader<R>) -> Vec<StationAggregate> {
    let mut results = HashMap::new();

    let mut buf = Vec::with_capacity(MAX_LINE_LEN);

    loop {
        let bytes_read = reader
            .read_until(b'\n', &mut buf)
            .expect("failed to read from BufReader");

        if bytes_read == 0 {
            break;
        }

        let mut split_idx = 0;
        loop {
            if buf[split_idx] == b';' {
                break;
            }
            split_idx += 1;
        }

        let name = &buf[..split_idx];
        let temp = custom_parse_float(&buf[split_idx + 1..bytes_read - 1]);

        let entry = results
            .entry(name.to_vec())
            .or_insert_with(StationAggregateTmp::default);

        if temp < entry.min {
            entry.min = temp;
        } else if temp > entry.max {
            entry.max = temp;
        }
        entry.total += temp;
        entry.count += 1;

        buf.clear();
    }

    let mut results = results
        .into_iter()
        .map(|(name, aggregate)| StationAggregate {
            name: unsafe { String::from_utf8_unchecked(name) },
            min: aggregate.min,
            max: aggregate.max,
            mean: aggregate.total / aggregate.count as f64,
        })
        .collect::<Vec<_>>();

    results.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    results
}

const USAGE_MSG: &str = "Usage: onebrc <data file path>";

pub fn parse_file_path() -> PathBuf {
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

#[inline]
pub fn custom_parse_float(bytes: &[u8]) -> f64 {
    let is_negative = bytes[0] == b'-';
    let bytes = if is_negative { &bytes[1..] } else { bytes };

    let mut period_idx = 0;
    loop {
        if bytes[period_idx] == b'.' {
            break;
        }
        period_idx += 1;
    }

    let mut i = 0;
    let mut result = 0;
    let mut base = 10u64.pow((period_idx - 1) as u32);
    while base >= 1 {
        result += (bytes[i] - b'0') as u64 * base;
        base /= 10;
        i += 1;
    }

    let decimal = (bytes[period_idx + 1] - b'0') as f64 / 10.0;

    if is_negative {
        -(result as f64) - decimal
    } else {
        result as f64 + decimal
    }
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

    #[test]
    fn can_parse_float_limited() {
        let input = b"99.9";
        let output = custom_parse_float(input);
        assert_eq!(output, 99.9);
    }
}
