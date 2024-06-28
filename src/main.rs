use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

fn main() {
    // println!();
    // let start = Instant::now();
    // count_lines_read_line(PathBuf::from_str("./data/measurements.txt").unwrap());
    // println!("count_lines_read_line: {:?}", start.elapsed());

    // let start = Instant::now();
    // count_lines_read_to_buf(PathBuf::from_str("./data/measurements.txt").unwrap());
    // println!("count_lines_read_to_buf: {:?}", start.elapsed());

    println!();
    let start = Instant::now();
    count_lines_concurrent(PathBuf::from_str("./data/measurements.txt").unwrap(), 32);
    println!("count_lines_concurrent: {:?}", start.elapsed());
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
