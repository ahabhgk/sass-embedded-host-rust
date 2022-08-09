#[path = "./helpers.rs"]
mod helpers;

use std::sync::Arc;

use helpers::{capture_stdio, exe_path, Sandbox, ToUrl};
use parking_lot::Mutex;
use sass_embedded_host_rust::{
  Logger, LoggerDebugOptions, LoggerWarnOptions, Options, OptionsBuilder,
  OutputStyle, Sass, Silent, StringOptions, StringOptionsBuilder, Syntax, Url,
};

#[test]
fn emits_debug_to_stderr_by_default() {
  let captured = capture_stdio(|| {
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string("@debug heck", StringOptions::default())
      .unwrap();
  });
  assert!(captured.out.is_empty());
  assert!(!captured.err.is_empty());
}

mod with_at_warn {
  use super::*;

  #[test]
  fn passes_the_message_and_stack_trace_to_the_logger() {
    #[derive(Debug)]
    struct MyLogger {
      count: Arc<Mutex<u8>>,
    }

    impl Logger for MyLogger {
      fn warn(&self, message: &str, options: &LoggerWarnOptions) {
        *self.count.lock() += 1;
        assert_eq!(message, "heck");
        assert!(options.span.is_none());
        assert!(options.stack.is_some());
        assert!(!options.deprecation);
      }
    }

    let count = Arc::new(Mutex::new(0));
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string(
        "@mixin foo {@warn heck}\n@include foo;",
        StringOptionsBuilder::default()
          .logger(MyLogger {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn stringifies_the_argument() {
    #[derive(Debug)]
    struct MyLogger {
      count: Arc<Mutex<u8>>,
    }

    impl Logger for MyLogger {
      fn warn(&self, message: &str, _: &LoggerWarnOptions) {
        *self.count.lock() += 1;
        assert_eq!(message, "#abc");
      }
    }

    let count = Arc::new(Mutex::new(0));
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string(
        "@warn #abc",
        StringOptionsBuilder::default()
          .logger(MyLogger {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn does_not_inspect_the_argument() {
    #[derive(Debug)]
    struct MyLogger {
      count: Arc<Mutex<u8>>,
    }

    impl Logger for MyLogger {
      fn warn(&self, message: &str, _: &LoggerWarnOptions) {
        *self.count.lock() += 1;
        assert_eq!(message, "");
      }
    }

    let count = Arc::new(Mutex::new(0));
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string(
        "@warn null",
        StringOptionsBuilder::default()
          .logger(MyLogger {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn emits_to_stderr_by_default() {
    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string("@warn heck", StringOptions::default())
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }

  #[test]
  fn does_not_emit_warnings_with_a_warn_callback() {
    #[derive(Debug)]
    struct MyLogger;

    impl Logger for MyLogger {
      fn warn(&self, _: &str, _: &LoggerWarnOptions) {}
    }

    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string(
          "@warn heck",
          StringOptionsBuilder::default().logger(MyLogger).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(captured.err.is_empty());
  }

  #[test]
  fn still_emits_warning_with_only_a_debug_callback() {
    #[derive(Debug)]
    struct MyLogger;

    impl Logger for MyLogger {
      fn debug(&self, _: &str, _: &LoggerDebugOptions) {}
    }

    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string(
          "@warn heck",
          StringOptionsBuilder::default().logger(MyLogger).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }

  #[test]
  fn does_not_emit_warnings_with_logger_silent() {
    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string(
          "@warn heck",
          StringOptionsBuilder::default().logger(Silent).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(captured.err.is_empty());
  }
}

mod with_at_debug {
  use super::*;

  #[test]
  fn passes_the_message_and_span_to_the_logger() {
    #[derive(Debug)]
    struct MyLogger {
      count: Arc<Mutex<u8>>,
    }

    impl Logger for MyLogger {
      fn debug(&self, message: &str, options: &LoggerDebugOptions) {
        *self.count.lock() += 1;
        let span = options.span.as_ref().unwrap();
        assert_eq!(message, "heck");
        assert_eq!(span.start.as_ref().unwrap().line, 0);
        assert_eq!(span.start.as_ref().unwrap().column, 0);
        assert_eq!(span.end.as_ref().unwrap().line, 0);
        assert_eq!(span.end.as_ref().unwrap().column, 11);
      }
    }

    let count = Arc::new(Mutex::new(0));
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string(
        "@debug heck",
        StringOptionsBuilder::default()
          .logger(MyLogger {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn stringifies_the_argument() {
    #[derive(Debug)]
    struct MyLogger {
      count: Arc<Mutex<u8>>,
    }

    impl Logger for MyLogger {
      fn debug(&self, message: &str, _: &LoggerDebugOptions) {
        *self.count.lock() += 1;
        assert_eq!(message, "#abc");
      }
    }

    let count = Arc::new(Mutex::new(0));
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string(
        "@debug #abc",
        StringOptionsBuilder::default()
          .logger(MyLogger {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn inspects_the_argument() {
    #[derive(Debug)]
    struct MyLogger {
      count: Arc<Mutex<u8>>,
    }

    impl Logger for MyLogger {
      fn debug(&self, message: &str, _: &LoggerDebugOptions) {
        *self.count.lock() += 1;
        assert_eq!(message, "null");
      }
    }

    let count = Arc::new(Mutex::new(0));
    let mut sass = Sass::new(exe_path());
    let _ = sass
      .compile_string(
        "@debug null",
        StringOptionsBuilder::default()
          .logger(MyLogger {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn emits_to_stderr_by_default() {
    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string("@debug heck", StringOptions::default())
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }

  #[test]
  fn does_not_emit_debugs_with_a_debug_callback() {
    #[derive(Debug)]
    struct MyLogger;

    impl Logger for MyLogger {
      fn debug(&self, _: &str, _: &LoggerDebugOptions) {}
    }

    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string(
          "@debug heck",
          StringOptionsBuilder::default().logger(MyLogger).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(captured.err.is_empty());
  }

  #[test]
  fn still_emits_debugs_with_only_a_warn_callback() {
    #[derive(Debug)]
    struct MyLogger;

    impl Logger for MyLogger {
      fn warn(&self, _: &str, _: &LoggerWarnOptions) {}
    }

    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string(
          "@debug heck",
          StringOptionsBuilder::default().logger(MyLogger).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }

  #[test]
  fn does_not_emit_debugs_with_logger_silent() {
    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile_string(
          "@debug heck",
          StringOptionsBuilder::default().logger(Silent).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(captured.err.is_empty());
  }
}

mod compile {
  use super::*;

  #[test]
  fn emits_to_stderr_by_default() {
    let sandbox = Sandbox::default();
    sandbox.write(
      sandbox.path().join("style.scss"),
      "@warn heck; @debug heck;",
    );

    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile(sandbox.path().join("style.scss"), Options::default())
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }

  #[test]
  fn does_not_emit_to_stderr_with_callbacks() {
    #[derive(Debug)]
    struct MyLogger;

    impl Logger for MyLogger {
      fn warn(&self, message: &str, _: &LoggerWarnOptions) {
        assert_eq!(message, "heck warn");
      }

      fn debug(&self, message: &str, _: &LoggerDebugOptions) {
        assert_eq!(message, "heck debug");
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(
      sandbox.path().join("style.scss"),
      "@warn heck warn; @debug heck debug",
    );

    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .compile(
          sandbox.path().join("style.scss"),
          OptionsBuilder::default().logger(MyLogger).build(),
        )
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(captured.err.is_empty());
  }
}
