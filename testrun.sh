./clean.sh
cargo build --release
cargo test --release
cargo run --release "testing" 5000