#![cfg(feature = "legacy")]

#[path = "./helpers.rs"]
mod helpers;

use std::{
  env,
  path::{Path, PathBuf},
  sync::Arc,
  time::SystemTime,
};

use helpers::{capture_stdio, exe_path, Sandbox};
use parking_lot::Mutex;
use sass_embedded_host_rust::{
  legacy::{
    IndentType, LegacyImporter, LegacyImporterResult, LegacyImporterThis,
    LegacyOptions, LegacyOptionsBuilder, LineFeed, OutputStyle, PATH_DELIMITER,
  },
  Exception, Result, Sass,
};

const SASS_PATH: &str = "SASS_PATH";

struct WithSassPathGuard(String);

impl Drop for WithSassPathGuard {
  fn drop(&mut self) {
    env::set_var(SASS_PATH, self.0.as_str());
  }
}

fn with_sass_path(paths: &[impl AsRef<Path>]) -> WithSassPathGuard {
  let old = env::var(SASS_PATH).unwrap_or_default();
  env::set_var(
    SASS_PATH,
    paths
      .iter()
      .map(|p| p.as_ref().to_str().unwrap())
      .collect::<Vec<_>>()
      .join(PATH_DELIMITER),
  );
  WithSassPathGuard(old)
}

mod render_sync {
  use super::*;

  #[test]
  fn one_of_data_and_file_must_be_set() {
    let mut sass = Sass::new(exe_path());
    assert!(sass.render(LegacyOptions::default()).is_err());
  }

  mod with_file {
    use super::*;

    #[test]
    fn renders_a_file() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    #[test]
    fn renders_a_file_from_a_relative_path() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");
      let _chdir = sandbox.chdir();

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    #[test]
    fn renders_a_file_with_the_indented_syntax() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.sass"), "a\n  b: c");
      let _chdir = sandbox.chdir();

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.sass"))
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    mod loads {
      use super::*;

      #[test]
      fn suppports_relative_imports_for_a_file() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("_other.scss"), "a {b: c}")
          .write(sandbox.path().join("importer.scss"), "@import \"other\";");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .file(sandbox.path().join("importer.scss"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
      }

      #[test]
      fn supports_relative_imports_for_a_file_from_a_relative_path() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("_other.scss"), "a {b: c}")
          .write(
            sandbox.path().join("subdir/importer.scss"),
            "@import \"../other\";",
          );
        let _chdir = sandbox.chdir();

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .file(sandbox.path().join("subdir/importer.scss"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
      }

      #[test]
      fn supports_absolute_path_imports() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("_other.scss"), "a {b: c}")
          .write(
            sandbox.path().join("importer.scss"),
            &format!(
              "@import \"{}\";",
              sandbox
                .path()
                .join("_other.scss")
                .to_str()
                .unwrap()
                .replace("\\", "\\\\")
            ),
          );
        let _chdir = sandbox.chdir();

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .file(sandbox.path().join("importer.scss"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
      }

      #[test]
      fn supports_import_only_files() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("_other.scss"), "a {b: regular}")
          .write(
            sandbox.path().join("_other.import.scss"),
            "a {b: import-only}",
          )
          .write(sandbox.path().join("importer.scss"), "@import \"other\";");
        let _chdir = sandbox.chdir();

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .file(sandbox.path().join("importer.scss"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: import-only;\n}".as_bytes());
      }

      #[test]
      fn supports_mixed_at_use_and_at_import() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("_other.scss"), "a {b: regular}")
          .write(
            sandbox.path().join("_other.import.scss"),
            "a {b: import-only}",
          )
          .write(
            sandbox.path().join("importer.scss"),
            "@use \"other\"; @import \"other\";",
          );
        let _chdir = sandbox.chdir();

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .file(sandbox.path().join("importer.scss"))
              .build(),
          )
          .unwrap();
        assert_eq!(
          res.css,
          "a {\n  b: regular;\n}\n\na {\n  b: import-only;\n}".as_bytes()
        );
      }
    }
  }

  mod with_data {
    use super::*;

    #[test]
    fn renders_a_string() {
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(LegacyOptionsBuilder::default().data("a {b: c}").build())
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    mod loads {
      use pathdiff::diff_paths;

      use super::*;

      #[test]
      fn supports_load_paths() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .data("@import \"test\"")
              .include_path(sandbox.path())
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
      }

      #[test]
      fn supports_sass_path() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("dir1/test1.scss"), "a {b: c}")
          .write(sandbox.path().join("dir2/test2.scss"), "x {y: z}");
        let _with_sass_path = with_sass_path(&[
          sandbox.path().join("dir1"),
          sandbox.path().join("dir2"),
        ]);

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .data("@import \"test1\"; @import \"test2\"")
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}\n\nx {\n  y: z;\n}".as_bytes());
      }

      #[test]
      fn load_paths_take_precedence_over_sass_path() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("dir1/test.scss"), "a {b: c}")
          .write(sandbox.path().join("dir2/test.scss"), "x {y: z}");
        let _with_sass_path = with_sass_path(&[sandbox.path().join("dir1")]);

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .data("@import \"test\"")
              .include_path(sandbox.path().join("dir2"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "x {\n  y: z;\n}".as_bytes());
      }

      #[test]
      fn a_file_imported_through_a_relative_load_path_supports_relative_imports(
      ) {
        let sandbox = Sandbox::default();
        sandbox
          .write(
            sandbox.path().join("sub/_midstream.scss"),
            "@import \"upstream\"",
          )
          .write(sandbox.path().join("sub/_upstream.scss"), "a {b: c}");
        let _with_sass_path = with_sass_path(&[sandbox.path().join("dir1")]);

        let mut sass = Sass::new(exe_path());
        let res = sass
          .render(
            LegacyOptionsBuilder::default()
              .data("@import \"sub/midstream\"")
              .include_path(
                diff_paths(sandbox.path(), env::current_dir().unwrap())
                  .unwrap(),
              )
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
      }
    }
  }

  mod with_both_data_and_file {
    use super::*;

    #[test]
    fn uses_the_data_parameter_as_a_source() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .data("x {y: z}")
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "x {\n  y: z;\n}".as_bytes());
    }

    #[test]
    fn does_not_require_the_file_path_to_exist() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("non-existent.scss"))
            .data("a {b: c}")
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    #[test]
    fn resolves_loads_relative_to_the_file_path_to_exist() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .data("@import \"other\"")
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }
  }

  #[test]
  fn resolves_meta_load_css_relative_to_the_containing_file() {
    let sandbox = Sandbox::default();
    sandbox
      .write(sandbox.path().join("sub/_upstream.scss"), "a {b: c}")
      .write(sandbox.path().join("sub/_midstream.scss"), "@use 'sass:meta';\n\n@mixin mixin {\n@include meta.load-css('upstream');\n}")
      .write(sandbox.path().join("downstream.scss"), "@use 'sub/midstream';\n\n@include midstream.mixin;");

    let mut sass = Sass::new(exe_path());
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("downstream.scss"))
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
  }
}

mod message {
  use super::*;

  #[test]
  fn resolves_meta_load_css_relative_to_the_containing_file() {
    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(LegacyOptionsBuilder::default().data("@warn heck").build())
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }

  #[test]
  fn emits_debug_messages_on_stderr_by_default() {
    let captured = capture_stdio(|| {
      let mut sass = Sass::new(exe_path());
      let _ = sass
        .render(LegacyOptionsBuilder::default().data("@debug heck").build())
        .unwrap();
    });
    assert!(captured.out.is_empty());
    assert!(!captured.err.is_empty());
  }
}

mod options {
  use super::*;

  mod indented_syntax {
    use super::*;

    #[test]
    fn renders_the_indented_syntax() {
      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("a\n  b: c")
            .indented_syntax(true)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }

    #[test]
    fn takes_precedence_over_the_file_extension() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a\n  b: c");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .indented_syntax(true)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }
  }

  mod output_style {
    use super::*;

    #[test]
    fn supports_the_expanded_output_style() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a\n  b: c");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("a {b: c}")
            .output_style(OutputStyle::Expanded)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}".as_bytes());
    }
  }
}
