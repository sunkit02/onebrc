use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

fn main() {
    let start = Instant::now();
    count_lines_read_to_buf(PathBuf::from_str("./data/measurements.txt").unwrap());
    println!("count_lines_read_to_buf: {:?}", start.elapsed());

    // let start = Instant::now();
    // count_lines_read_line(PathBuf::from_str("./data/measurements.txt").unwrap());
    // println!("count_lines_read_line: {:?}", start.elapsed());
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
        lines += buf[..read]
            .iter()
            .filter(|&&byte| byte == '\n' as u8)
            .count();
    }

    println!("{lines}");
}

fn count_lines_read_line(path: PathBuf) {
    let f = File::open(path).unwrap();
    let reader = BufReader::with_capacity(3 * 1024, f);
    let lines = reader.lines().count();

    println!("{lines}");
}
