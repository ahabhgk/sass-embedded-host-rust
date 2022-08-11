#![cfg(feature = "legacy")]

#[path = "helpers.rs"]
mod helpers;

use std::{env, path::PathBuf, sync::Arc, time::SystemTime};

use helpers::{exe_path, Sandbox};
use parking_lot::Mutex;
use sass_embedded_host_rust::{
  legacy::{
    IndentType, LegacyImporter, LegacyImporterResult, LegacyImporterThis,
    LegacyOptionsBuilder, LineFeed, PATH_DELIMITER,
  },
  Exception, Result, Sass,
};

#[test]
fn imports_cascade_through_importers() {
  #[derive(Debug, Default)]
  struct FooImporter;

  impl LegacyImporter for FooImporter {
    fn call(
      &self,
      _: &LegacyImporterThis,
      url: &str,
      _: &str,
    ) -> Result<Option<LegacyImporterResult>> {
      if url == "foo" {
        return Ok(Some(LegacyImporterResult::contents("@import \"bar\"")));
      }
      Ok(None)
    }
  }

  #[derive(Debug, Default)]
  struct BarImporter;

  impl LegacyImporter for BarImporter {
    fn call(
      &self,
      _: &LegacyImporterThis,
      url: &str,
      _: &str,
    ) -> Result<Option<LegacyImporterResult>> {
      if url == "bar" {
        return Ok(Some(LegacyImporterResult::contents("@import \"baz\"")));
      }
      Ok(None)
    }
  }

  #[derive(Debug, Default)]
  struct BazImporter;

  impl LegacyImporter for BazImporter {
    fn call(
      &self,
      _: &LegacyImporterThis,
      url: &str,
      _: &str,
    ) -> Result<Option<LegacyImporterResult>> {
      if url == "baz" {
        return Ok(Some(LegacyImporterResult::contents("a {b: c}")));
      }
      Ok(None)
    }
  }

  let mut sass = Sass::new(exe_path());
  let res = sass
    .render(
      LegacyOptionsBuilder::default()
        .data("@import 'foo'")
        .importer(FooImporter)
        .importer(BarImporter)
        .importer(BazImporter)
        .build(),
    )
    .unwrap();
  assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
}

#[test]
fn an_empty_object_means_an_empty_file() {
  #[derive(Debug, Default)]
  struct EmptyImporter;

  impl LegacyImporter for EmptyImporter {
    fn call(
      &self,
      _: &LegacyImporterThis,
      url: &str,
      _: &str,
    ) -> Result<Option<LegacyImporterResult>> {
      if url == "foo" {
        return Ok(Some(LegacyImporterResult::contents(String::new())));
      }
      Ok(None)
    }
  }

  let mut sass = Sass::new(exe_path());
  let res = sass
    .render(
      LegacyOptionsBuilder::default()
        .data("@import 'foo'")
        .importer(EmptyImporter)
        .build(),
    )
    .unwrap();
  assert_eq!(res.css, "".as_bytes());
}

mod import_precedence {
  use super::*;

  mod in_sandbox_dir {
    use super::*;

    #[test]
    fn relative_file_is_sharp_1() {
      #[derive(Debug, Default)]
      struct MyImporter;

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          _: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          Ok(Some(LegacyImporterResult::contents("a {from: importer}")))
        }
      }

      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("sub/test.scss"), "a {from: relative}")
        .write(sandbox.path().join("sub/base.scss"), "@import \"test\"");
      let _chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("sub/base.scss"))
            .importer(MyImporter)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: relative;\n}".as_bytes());
    }

    #[test]
    fn importer_is_sharp_2() {
      #[derive(Debug, Default)]
      struct MyImporter;

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          _: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          Ok(Some(LegacyImporterResult::contents("a {from: importer}")))
        }
      }

      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {from: cwd}");
      let _chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"test\"")
            .importer(MyImporter)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: importer;\n}".as_bytes());
    }

    #[test]
    fn cwd_is_sharp_3() {
      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("test.scss"), "a {from: cwd}")
        .write(sandbox.path().join("sub/test.scss"), "a {from: load path}");
      let _chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"test\"")
            .include_path(sandbox.path().join("sub"))
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: cwd;\n}".as_bytes());
    }

    // Regression test for embedded host.
    #[test]
    fn falls_back_to_load_path_if_imports_list_is_empty() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {from: load path}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"test\"")
            .include_path(sandbox.path())
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: load path;\n}".as_bytes());
    }
  }
}

mod with_contents {
  use super::*;

  #[test]
  fn imports_a_file_by_contents() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::contents("a {b: c}")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn contents_take_precedence_over_file_name() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::both(
          "a {from: contents}",
          self.sandbox.path().join("test.scss"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {from: path}");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"test\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  from: contents;\n}".as_bytes());
  }

  #[test]
  fn contents_use_file_name_as_canonical_url() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::both("", "bar")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
    assert!(res.stats.included_files.contains(&"bar".to_owned()));
  }

  // Regression test for sass/dart-sass#1410.
  #[test]
  fn passes_through_an_absolute_file_path() {
    #[derive(Debug, Default)]
    struct MyImporter {
      file: PathBuf,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::both("", self.file.clone())))
      }
    }

    let file = env::current_dir().unwrap().join("bar");
    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { file: file.clone() })
          .build(),
      )
      .unwrap();
    assert!(res
      .stats
      .included_files
      .contains(&file.to_str().unwrap().to_string()));
  }
}

mod with_a_file_redirect {
  use super::*;

  #[test]
  fn imports_the_chosen_file() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("test.scss"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn supports_the_indented_syntax() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("test.sass"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.sass"), "a\n  b: c");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn supports_plain_css() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("test.css"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.css"), "@import \"bar\"");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "@import \"bar\";".as_bytes());
  }

  #[test]
  fn supports_partials() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("target.scss"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_target.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn supports_import_only_files() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("target.scss"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(sandbox.path().join("target.scss"), "a {b: regular}")
      .write(
        sandbox.path().join("target.import.scss"),
        "a {b: import-only}",
      );

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: import-only;\n}".as_bytes());
  }

  #[test]
  fn supports_mixed_at_use_and_at_import() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("target.scss"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(sandbox.path().join("target.scss"), "a {b: regular}")
      .write(
        sandbox.path().join("target.import.scss"),
        "a {b: import-only}",
      );

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@use \"foo\"; @import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(
      res.css,
      "a {\n  b: regular;\n}\n\na {\n  b: import-only;\n}".as_bytes()
    );
  }

  #[test]
  fn may_be_extensionless() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file(
          self.sandbox.path().join("test"),
        )))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn is_resolved_relative_to_the_base_file() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file("_other.scss")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(sandbox.path().join("_other.scss"), "a {b: c}")
      .write(sandbox.path().join("test.scss"), "@import \"foo\"");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn puts_the_absolute_path_in_included_files() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file("_other.scss")))
      }
    }

    let sandbox = Sandbox::default();
    let other = sandbox.path().join("_other.scss");
    let test = sandbox.path().join("test.scss");
    sandbox
      .write(other.clone(), "a {b: c}")
      .write(test.clone(), "@import \"foo\"");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
    assert!(res
      .stats
      .included_files
      .contains(&test.to_str().unwrap().to_string()));
    assert!(res
      .stats
      .included_files
      .contains(&other.to_str().unwrap().to_string()));
  }

  #[test]
  fn is_resolved_relative_to_include_paths() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file("test.scss")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .include_path(sandbox.path())
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }

  #[test]
  fn relative_to_the_base_file_takes_precedence_over_include_paths() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file("_other.scss")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(sandbox.path().join("test.scss"), "@import \"foo\"")
      .write(sandbox.path().join("_other.scss"), "a {from: relative}")
      .write(
        sandbox.path().join("sub/_other.scss"),
        "a {from: load path}",
      );

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .include_path(sandbox.path().join("sub"))
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  from: relative;\n}".as_bytes());
  }

  mod in_the_sandbox_directory {
    use super::*;

    #[test]
    fn is_resolved_relative_to_the_cwd() {
      #[derive(Debug, Default)]
      struct MyImporter;

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          _: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          Ok(Some(LegacyImporterResult::file("test.scss")))
        }
      }

      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");
      let _chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"foo\"")
            .importer(MyImporter)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    #[test]
    fn file_relative_takes_precedence_over_the_cwd() {
      #[derive(Debug, Default)]
      struct MyImporter;

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          _: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          Ok(Some(LegacyImporterResult::file("_other.scss")))
        }
      }

      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("_other.scss"), "a {from: cwd}")
        .write(sandbox.path().join("sub/test.scss"), "@import \"foo\"")
        .write(sandbox.path().join("sub/_other.scss"), "a {from: relative}");
      let _chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("sub/test.scss"))
            .importer(MyImporter)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: relative;\n}".as_bytes());
    }

    #[test]
    fn the_cwd_takes_precedence_over_include_paths() {
      #[derive(Debug, Default)]
      struct MyImporter;

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          _: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          Ok(Some(LegacyImporterResult::file("_other.scss")))
        }
      }

      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("_other.scss"), "a {from: cwd}")
        .write(sandbox.path().join("test.scss"), "@import \"foo\"")
        .write(
          sandbox.path().join("sub/_other.scss"),
          "a {from: load path}",
        );
      let _chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .include_path(sandbox.path().join("sub"))
            .importer(MyImporter)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: cwd;\n}".as_bytes());
    }
  }
}

mod the_imported_url {
  use super::*;

  #[test]
  fn is_the_exact_imported_text() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(url, "foo");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn is_not_resolved_relative_to_the_current_file() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        if url == "foo/bar" {
          return Ok(Some(LegacyImporterResult::contents("@import \"baz\"")));
        }
        assert_eq!(url, "baz");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo/bar\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 2);
  }

  #[test]
  fn is_added_to_included_files() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap();
  }

  // Regression test for sass/dart-sass#1137.
  #[test]
  fn is_not_changed_if_it_is_root_relative_with_no_nesting() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(url, "/foo");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"/foo\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  // Regression test for sass/embedded-host-node#1137.
  #[test]
  fn is_not_changed_if_it_is_root_relative_with_nesting() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(url, "/foo/bar/baz");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"/foo/bar/baz\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn is_converted_to_a_file_url_if_it_is_an_absolute_windows_path() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(url, "file:///C:/foo");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"C:/foo\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }
}

mod the_previous_url {
  use super::*;

  #[test]
  fn is_an_absolute_path_for_stylesheets_from_the_filesystem() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        prev: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(
          prev,
          env::current_dir()
            .unwrap()
            .join(self.sandbox.path().join("test.scss"))
            .to_str()
            .unwrap()
        );
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "@import \"foo\"");

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .importer(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn is_an_absolute_path_for_stylesheets_redirected_to_the_filesystem() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        prev: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        if url == "foo" {
          return Ok(Some(LegacyImporterResult::file("_other.scss")));
        }
        assert_eq!(url, "baz");
        assert_eq!(
          prev,
          self.sandbox.path().join("_other.scss").to_str().unwrap()
        );
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(sandbox.path().join("test.scss"), "@import \"foo\"")
      .write(sandbox.path().join("_other.scss"), "@import \"baz\"");

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .importer(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 2);
  }

  #[test]
  fn is_stdin_for_string_stylesheets() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        prev: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(prev, "stdin");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn is_the_imported_string_for_imports_from_importers() {
    #[derive(Debug, Default)]
    struct MyImporter1 {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter1 {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        Ok(if url == "foo" {
          Some(LegacyImporterResult::contents("@import \"bar\""))
        } else {
          None
        })
      }
    }

    #[derive(Debug, Default)]
    struct MyImporter2 {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter2 {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        prev: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(url, "bar");
        assert_eq!(prev, "foo");
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count1 = Arc::new(Mutex::new(0));
    let count2 = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter1 {
            count: Arc::clone(&count1),
          })
          .importer(MyImporter2 {
            count: Arc::clone(&count2),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count1.lock(), 2);
    assert_eq!(*count2.lock(), 1);
  }

  // Regression test for sass/embedded-host-node#120
  #[test]
  fn is_passed_after_a_relative_import() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        url: &str,
        prev: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert_eq!(url, "importer");
        assert_eq!(
          prev,
          self.sandbox.path().join("test.scss").to_str().unwrap()
        );
        Ok(Some(LegacyImporterResult::contents("a {b: importer}")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(
        sandbox.path().join("test.scss"),
        "@import \"relative\";\n@import \"importer\";",
      )
      .write(sandbox.path().join("_relative.scss"), "a {b: relative}");

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .importer(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(
      res.css,
      "a {\n  b: relative;\n}\n\na {\n  b: importer;\n}".as_bytes()
    );
    assert_eq!(*count.lock(), 1);
  }
}

mod this {
  use super::*;

  #[test]
  fn includes_default_option_values() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        this: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        let options = &this.options;
        assert_eq!(
          options.include_paths,
          std::env::current_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(options.precision, 10);
        assert_eq!(options.style, 1);
        assert_eq!(options.indent_type, IndentType::Space);
        assert_eq!(options.indent_width, 2);
        assert_eq!(options.linefeed, LineFeed::LF);
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn includes_the_data_when_rendering_via_data() {
    #[derive(Debug, Default)]
    struct MyImporter {
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        this: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        let options = &this.options;
        assert!(options.data.is_some());
        assert_eq!(options.data.as_ref().unwrap(), "@import \"foo\"");
        assert!(options.file.is_none());
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter {
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn includes_the_filename_when_rendering_via_file() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        this: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        let options = &this.options;
        assert!(options.file.is_some());
        assert_eq!(
          options.file.as_ref().unwrap(),
          &self.sandbox.path().join("test.scss")
        );
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "@import \"foo\"");

    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .importer(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  #[test]
  fn includes_other_include_paths() {
    #[derive(Debug, Default)]
    struct MyImporter {
      sandbox: Sandbox,
      count: Arc<Mutex<u8>>,
    }

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        this: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        *self.count.lock() += 1;
        assert!(this.options.include_paths.contains(&format!(
          "{}{PATH_DELIMITER}{}",
          std::env::current_dir().unwrap().to_str().unwrap(),
          self.sandbox.path().to_str().unwrap(),
        )));
        Ok(Some(LegacyImporterResult::contents("")))
      }
    }

    let sandbox = Sandbox::default();
    let root = sandbox.path().to_str().unwrap().to_string();
    let mut sass = Sass::new(exe_path());
    let count = Arc::new(Mutex::new(0));
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          })
          .include_path(root)
          .build(),
      )
      .unwrap();
    assert_eq!(*count.lock(), 1);
  }

  mod includes_render_stats_with {
    use super::*;

    #[test]
    fn a_start_time() {
      #[derive(Debug)]
      struct MyImporter {
        start: SystemTime,
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert!(this.options.result.stats.start > self.start);
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let start = SystemTime::now();
      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"foo\"")
            .importer(MyImporter {
              start,
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }

    #[test]
    fn a_data_entry() {
      #[derive(Debug)]
      struct MyImporter {
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert_eq!(this.options.result.stats.entry, "data");
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"foo\"")
            .importer(MyImporter {
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }

    #[test]
    fn a_file_entry() {
      #[derive(Debug)]
      struct MyImporter {
        sandbox: Sandbox,
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert_eq!(
            this.options.result.stats.entry,
            self.sandbox.path().join("test.scss").to_str().unwrap()
          );
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "@import \"foo\"");

      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .importer(MyImporter {
              sandbox,
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }
  }

  mod includes_a_from_import_field_that_is {
    use super::*;

    #[test]
    fn true_for_an_at_import() {
      #[derive(Debug)]
      struct MyImporter {
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert!(this.from_import);
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"foo\"")
            .importer(MyImporter {
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }

    #[test]
    fn false_for_an_at_use() {
      #[derive(Debug)]
      struct MyImporter {
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert!(!this.from_import);
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@use \"foo\"")
            .importer(MyImporter {
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }

    #[test]
    fn false_for_an_at_forward() {
      #[derive(Debug)]
      struct MyImporter {
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert!(!this.from_import);
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@forward \"foo\"")
            .importer(MyImporter {
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }

    #[test]
    fn false_for_meta_load_css() {
      #[derive(Debug)]
      struct MyImporter {
        count: Arc<Mutex<u8>>,
      }

      impl LegacyImporter for MyImporter {
        fn call(
          &self,
          this: &LegacyImporterThis,
          _: &str,
          _: &str,
        ) -> Result<Option<LegacyImporterResult>> {
          *self.count.lock() += 1;
          assert!(!this.from_import);
          Ok(Some(LegacyImporterResult::contents("")))
        }
      }

      let count = Arc::new(Mutex::new(0));
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@use \"sass:meta\"; @include meta.load-css(\"foo\")")
            .importer(MyImporter {
              count: Arc::clone(&count),
            })
            .build(),
        )
        .unwrap();
      assert_eq!(*count.lock(), 1);
    }
  }
}

mod gracefully_handles_an_error_when {
  use super::*;

  #[test]
  fn an_importer_redirects_to_a_non_existent_file() {
    #[derive(Debug)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::file("_dose_not_exist")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let err = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
  }

  #[test]
  fn an_error_is_returned() {
    #[derive(Debug)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Err(Exception::new("oh no"))
      }
    }

    let mut sass = Sass::new(exe_path());
    let err = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
  }

  #[test]
  fn null_is_returned() {
    #[derive(Debug)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(None)
      }
    }

    let mut sass = Sass::new(exe_path());
    let err = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
  }

  #[test]
  fn it_occurs_in_a_file_with_a_custom_url_scheme() {
    #[derive(Debug)]
    struct MyImporter;

    impl LegacyImporter for MyImporter {
      fn call(
        &self,
        _: &LegacyImporterThis,
        _: &str,
        _: &str,
      ) -> Result<Option<LegacyImporterResult>> {
        Ok(Some(LegacyImporterResult::contents("@error \"oh no\"")))
      }
    }

    let mut sass = Sass::new(exe_path());
    let err = sass
      .render(
        LegacyOptionsBuilder::default()
          .data("@import \"foo:bar\"")
          .importer(MyImporter)
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
    assert_eq!(err.span().unwrap().url, "foo:bar");
  }
}
