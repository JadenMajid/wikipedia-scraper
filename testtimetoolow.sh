./clean.sh
cargo build --release
cargo test --release
cargo run --release "testing" 4999 