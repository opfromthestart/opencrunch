# OpenCrunch
A statistics program made for college stats.

Works for calculating area under Normal, Chi squared, T, and F distributions.
Calculate the probability of a sample mean from a finite or infinite population.

## Usage
For x values and other ranges:
`>1.0` and `>=1.0` calculates the area from 1 to infinity.
`<1.0` and `<=1.0` calculates the area from negative infinity to 1.
`[1.0,2.0]` calculates the area from 1 to 2.
`]1.0,2.0[` calculates the area from negative infinity to 1 and from 2 to infinity.

## Building
Just do `cargo run`.
For web, do 
```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./web/ target/wasm32-unknown-unknown/release/opencrunch.wasm --no-typescript --no-modules
```
Then open the index.html file in the web folder.

## Contributing
Anything is cool. Be at least mildly desriptive with commits.
