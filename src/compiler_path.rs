use std::env;

pub fn compiler_path() -> Option<String> {
  // env::var("SASS_EMBEDDED_PATH").ok()
  Some(String::from("/Users/bytedance/Codes/sass-embedded-host-rust/dart-sass-embedded/dart-sass-embedded"))
}
