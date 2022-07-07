use std::env;

pub fn compiler_path() -> Option<String> {
  env::var("SASS_EMBEDDED_PATH").ok()
}
