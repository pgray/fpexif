we must always run the following before pushing our branches

- `nice -n 19 cargo test`
- `nice -n 19 cargo fmt`
- `nice -n 19 cargo clippy`
