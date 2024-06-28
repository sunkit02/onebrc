# 1 Billion Row Challenge in Rust

This is an attempt at the [one billion row challenge](https://github.com/gunnarmorling/1brc) but in rust.

All the following attempts were ran on my personal laptop, a Dell XPS 15 9510,
in AC mode and with the following specs:

- OS: Arch Linux x86_64, kernel: 6.6.32-1-lts
- CPU: Intel Core i7-11800H
- RAM: 32 GB

## Attempts

The following are the different iterations of the parsing function.

### Naive Baseline (time: ~106 secs)

This approach is to parse the file using the naive approach of reading each
line of the file through a `BufReader` using the built in `lines` method and
splitting each line at ';' then parsing each half individually without any custom
parsing. (This is clearly very slow, just counting the lines using a similar
approach takes ~54 secs)

```rust
const BUFFER_SIZE: usize = 3 * 1024;

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
```

### Increasing the buffer size to 1MB (time: ~104 secs)

My initial thought is that since reading is taking so long, whether increasing
the buffer size dramatically will decrease the time spend on IO. There was some
improvement of the overall runtime (~2 secs) but it's not as significant as I
thought it would be.

```rust
const BUFFER_SIZE: usize = 1024 * 1024;
```

### Custom parsing of station name (time: ~88 secs)

The idea here is to minimize the overhead caused by ut8 string validation and
unnecessary copying of the station name. So instead of using the convenient
`lines` method on `BufReader` which produces an iterator over '\n' delimited
lines, I am using the `read_until` '\n' to avoid the bytes to ut8 string conversion
overhead. I also avoided copying the station name on each `HashMap` access
and only doing the copy when encountering the station name for the first time.

```rust
fn parse_lines<R: Read>(mut reader: BufReader<R>) -> Vec<StationAggregate> {
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
        let temp = String::from_utf8_lossy(&buf[split_idx + 1..bytes_read - 1])
            .parse()
            .expect("failed to parse temp");

        if !results.contains_key(name.as_ref()) {
            results.insert(
                name.to_vec(),
                StationAggregateTmp {
                    min: temp,
                    max: temp,
                    total: 0f64,
                    count: 0,
                },
            );
        }

        let entry = results.get_mut(name).unwrap();

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
```

### Custom parsing (very bad) of floats (time: ~79 secs)

Since we know that the maximum and minimum values of the temperature for all
entries, we can create a custom parsing function for it. Here I have created a
very bad implementation of it but it still managed to sqeeze out a bit of extra
performance.

```rust
#[inline]
fn parse_float_limited(bytes: &[u8]) -> f64 {
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

pub fn parse_lines<R: Read>(mut reader: BufReader<R>) -> Vec<StationAggregate> {
    <unchanged>
        let temp = parse_float_limited(&buf[split_idx + 1..bytes_read - 1]);
    <unchanded>
}
```
