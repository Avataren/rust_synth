unit tests: cargo test -- --nocapture
benchmark: cargo bench

native:
cargo run --release

to start web:
cd web
npm install
npm run start:release
