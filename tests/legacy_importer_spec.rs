#![cfg(feature = "legacy")]

#[path = "./helpers.rs"]
mod helpers;

use std::{env, path::PathBuf, sync::Arc};

use helpers::{exe_path, Sandbox, ToUrl};
use parking_lot::Mutex;
use sass_embedded_host_rust::{
  legacy::{
    LegacyImporter, LegacyImporterResult, LegacyImporterThis,
    LegacyOptionsBuilder,
  },
  Options, OptionsBuilder, OutputStyle, Result, Sass, StringOptions,
  StringOptionsBuilder, Syntax, Url,
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
        .importer(Box::new(FooImporter) as Box<dyn LegacyImporter>)
        .importer(Box::new(BarImporter) as Box<dyn LegacyImporter>)
        .importer(Box::new(BazImporter) as Box<dyn LegacyImporter>)
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
        .importer(Box::new(EmptyImporter) as Box<dyn LegacyImporter>)
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
      let chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("sub/base.scss").to_string_lossy())
            .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: relative;\n}".as_bytes());
      drop(chdir);
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
      let chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"test\"")
            .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: importer;\n}".as_bytes());
      drop(chdir);
    }

    #[test]
    fn cwd_is_sharp_3() {
      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("test.scss"), "a {from: cwd}")
        .write(sandbox.path().join("sub/test.scss"), "a {from: load path}");
      let chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"test\"")
            .include_path(sandbox.path().join("sub").to_string_lossy())
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: cwd;\n}".as_bytes());
      drop(chdir);
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
            .include_path(sandbox.path().to_string_lossy())
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
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { file: file.clone() })
            as Box<dyn LegacyImporter>)
          .build(),
      )
      .unwrap();
    assert!(res
      .stats
      .included_files
      .contains(&file.to_string_lossy().to_string()));
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter { sandbox }) as Box<dyn LegacyImporter>)
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
          .file(sandbox.path().join("test.scss").to_string_lossy())
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
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
          .file(sandbox.path().join("test.scss").to_string_lossy())
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
          .build(),
      )
      .unwrap();
    assert!(res
      .stats
      .included_files
      .contains(&test.to_string_lossy().to_string()));
    assert!(res
      .stats
      .included_files
      .contains(&other.to_string_lossy().to_string()));
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
          .include_path(sandbox.path().to_string_lossy())
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
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
          .file(sandbox.path().join("test.scss").to_string_lossy())
          .include_path(sandbox.path().join("sub").to_string_lossy())
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
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
      let chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("@import \"foo\"")
            .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
      drop(chdir);
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
      let chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("sub/test.scss").to_string_lossy())
            .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: relative;\n}".as_bytes());
      drop(chdir);
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
      let chdir = sandbox.chdir();
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss").to_string_lossy())
            .include_path(sandbox.path().join("sub").to_string_lossy())
            .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  from: cwd;\n}".as_bytes());
      drop(chdir);
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
          .importer(Box::new(MyImporter {
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter {
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter {
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter {
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter {
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
            .to_string_lossy()
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
          .file(sandbox.path().join("test.scss").to_string_lossy())
          .importer(Box::new(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          self.sandbox.path().join("_other.scss").to_string_lossy()
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
          .file(sandbox.path().join("test.scss").to_string_lossy())
          .importer(Box::new(MyImporter {
            sandbox,
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter {
            count: Arc::clone(&count),
          }) as Box<dyn LegacyImporter>)
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
          .importer(Box::new(MyImporter1 {
            count: Arc::clone(&count1),
          }) as Box<dyn LegacyImporter>)
          .importer(Box::new(MyImporter2 {
            count: Arc::clone(&count2),
          }) as Box<dyn LegacyImporter>)
          .build(),
      )
      .unwrap();
    assert_eq!(*count1.lock(), 2);
    assert_eq!(*count2.lock(), 1);
  }
}
