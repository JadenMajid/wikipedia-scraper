./clean.sh
cargo build --release
cargo test --release
cargo run --release "ACE_Encrypt" 10000
