# OpenCrunch
A statistics program made for college stats.

## Building
Just do `cargo run`.
For web, do 
```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./web/ --target web ./target/wasm32-unknown-unknown/release/opencrunch.wasm --no-typescript
wasm-bindgen --out-dir ./web/ target/wasm32-unknown-unknown/release/opencrunch.wasm --no-typescript --no-modules
```

## Contributing
Anything is cool. Be at least mildly desriptive with commits.