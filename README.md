# sass-embedded-host-rust

🦀 A Rust library that will communicate with [Embedded Dart Sass](https://github.com/sass/dart-sass-embedded) using the [Embedded Sass protocol](https://github.com/sass/embedded-protocol).

```rust
use sass_embedded::{Sass, StringOptions};

let mut sass = Sass::new("path/to/sass_embedded").unwrap();
let res = sass.compile_string("a {b: c}", StringOptions::default()).unwrap();
println!("{:?}", res);
```

For more details, checkout [docs.rs](https://docs.rs/sass-embedded) and [examples](https://github.com/ahabhgk/sass-embedded-host-rust/tree/main/examples).

## Contributing

1. Install the [Protocol Buffer Compiler](https://grpc.io/docs/protoc-installation/).
2. Run `npm install && npm run setup`.
3. Find issues and welcome PRs.
