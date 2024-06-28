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
