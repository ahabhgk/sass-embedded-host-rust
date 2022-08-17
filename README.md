# sass-embedded-host-rust

🦀 A Rust library that will communicate with [Embedded Dart Sass](https://github.com/sass/dart-sass-embedded) using the [Embedded Sass protocol](https://github.com/sass/embedded-protocol).

```rust
use sass_embedded::{Sass, StringOptions};

let mut sass = Sass::new("path/to/sass_embedded").unwrap();
let res = sass.compile_string("a {b: c}", StringOptions::default()).unwrap();
println!("{:?}", res);
```

For more details, checkout the [docs.rs](https://docs.rs/sass-embedded)

## Benchmark

[Compile bootstrap with `sass-embedded-host-rust`, `sass-embedded-host-node`, and `dart-sass`](https://github.com/ahabhgk/sass-embedded-host-rust/tree/main/benches/bootstrap)

```bash
$ cargo bench bootstrap
Benchmarking bootstrap/Host Rust: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 5.6s.
bootstrap/Host Rust     time:   [502.81 ms 507.54 ms 512.59 ms]                              
                        change: [-14.057% -9.4932% -5.7950%] (p = 0.00 < 0.05)
                        Performance has improved.
Benchmarking bootstrap/Host Node: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 14.7s.
bootstrap/Host Node     time:   [1.1295 s 1.2127 s 1.3098 s]                                 
                        change: [+1.1844% +9.3527% +18.371%] (p = 0.05 > 0.05)
                        No change in performance detected.
Benchmarking bootstrap/Dart Sass: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 24.8s.
bootstrap/Dart Sass     time:   [2.4472 s 2.4728 s 2.4986 s]                                 
                        change: [-1.1681% +1.3988% +3.8141%] (p = 0.31 > 0.05)
                        No change in performance detected.
```

## Contributing

1. Install the [Protocol Buffer Compiler](https://grpc.io/docs/protoc-installation/).
2. Run `cd scripts && npm install && node setup.mjs`.
3. Find issues and welcome PRs.
