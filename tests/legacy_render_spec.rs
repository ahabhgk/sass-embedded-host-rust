#![cfg(feature = "legacy")]

#[path = "helpers.rs"]
mod helpers;

use std::{env, path::Path};

use helpers::{capture_stdio, exe_path, Sandbox, ToUrl};
use sass_embedded::{
  legacy::{LegacyOptions, LegacyOptionsBuilder, OutputStyle, PATH_DELIMITER},
  Sass,
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
    let mut sass = Sass::new(exe_path()).unwrap();
    assert!(sass.render(LegacyOptions::default()).is_err());
  }

  mod with_file {
    use super::*;

    #[test]
    fn renders_a_file() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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
                .replace('\\', "\\\\")
            ),
          );
        let _chdir = sandbox.chdir();

        let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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
      let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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

        let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

    let mut sass = Sass::new(exe_path()).unwrap();
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
      let mut sass = Sass::new(exe_path()).unwrap();
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
      let mut sass = Sass::new(exe_path()).unwrap();
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
      let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

      let mut sass = Sass::new(exe_path()).unwrap();
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

    #[test]
    fn supports_the_compressed_output_style() {
      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .data("a {b: c}")
            .output_style(OutputStyle::Compressed)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a{b:c}".as_bytes());
    }
  }

  mod quiet_deps {
    use super::*;

    mod in_a_relative_load_from_the_entrypoint {
      use super::*;

      #[test]
      fn emits_at_warn() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("_other.scss"), "@warn heck");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(!captured.err.is_empty());
      }

      #[test]
      fn emits_at_debug() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("_other.scss"), "@debug heck");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.contains("heck"));
      }

      #[test]
      fn emits_parser_warnings() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("_other.scss"), "a {b: c && d}");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.contains("&&"));
      }

      #[test]
      fn emits_evaluation_warnings() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("_other.scss"), "#{blue} {b: c}");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.contains("blue"));
      }
    }

    mod in_a_load_path_load {
      use super::*;

      #[test]
      fn emits_at_warn() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("dir/_other.scss"), "@warn heck");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .include_path(sandbox.path().join("dir"))
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.contains("heck"));
      }

      #[test]
      fn emits_at_debug() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("dir/_other.scss"), "@debug heck");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .include_path(sandbox.path().join("dir"))
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.contains("heck"));
      }

      #[test]
      fn emits_parser_warnings() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("dir/_other.scss"), "a {b: c && d}");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .include_path(sandbox.path().join("dir"))
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.is_empty());
      }

      #[test]
      fn emits_evaluation_warnings() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("test.scss"), "@use \"other\"")
          .write(sandbox.path().join("dir/_other.scss"), "#{blue} {b: c}");

        let captured = capture_stdio(|| {
          let mut sass = Sass::new(exe_path()).unwrap();
          let _ = sass
            .render(
              LegacyOptionsBuilder::default()
                .file(sandbox.path().join("test.scss"))
                .quiet_deps(true)
                .include_path(sandbox.path().join("dir"))
                .build(),
            )
            .unwrap();
        });
        assert!(captured.out.is_empty());
        assert!(captured.err.is_empty());
      }
    }
  }

  mod verbose {
    use super::*;

    const DATA: &str = r#"
      $_: call("inspect", null);
      $_: call("rgb", 0, 0, 0);
      $_: call("nth", null, 1);
      $_: call("join", null, null);
      $_: call("if", true, 1, 2);
      $_: call("hsl", 0, 100%, 100%);
      $_: 1/2;
      $_: 1/3;
      $_: 1/4;
      $_: 1/5;
      $_: 1/6;
      $_: 1/7;
    "#;

    #[test]
    fn when_it_is_true_prints_all_deprecation_warnings() {
      let captured = capture_stdio(|| {
        let mut sass = Sass::new(exe_path()).unwrap();
        let _ = sass
          .render(
            LegacyOptionsBuilder::default()
              .data(DATA)
              .verbose(true)
              .build(),
          )
          .unwrap();
      });

      assert!(captured.out.is_empty());
      assert!(captured.err.matches("call()").count() == 6);
      assert!(captured.err.matches("math.div").count() == 6);
    }

    #[test]
    fn when_it_is_false_prints_only_five_of_each_deprecation_warning() {
      let captured = capture_stdio(|| {
        let mut sass = Sass::new(exe_path()).unwrap();
        let _ = sass
          .render(LegacyOptionsBuilder::default().data(DATA).build())
          .unwrap();
      });

      assert!(captured.out.is_empty());
      assert!(captured.err.matches("call()").count() == 5);
      assert!(captured.err.matches("math.div").count() == 5);
    }
  }
}

mod the_result_object {
  use super::*;

  #[test]
  fn includes_the_filename() {
    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .build(),
      )
      .unwrap();
    assert_eq!(
      res.stats.entry,
      sandbox.path().join("test.scss").to_str().unwrap(),
    );
  }

  #[test]
  fn includes_data_without_a_filename() {
    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .render(LegacyOptionsBuilder::default().data("a {b: c}").build())
      .unwrap();
    assert_eq!(res.stats.entry, "data");
  }

  #[test]
  fn includes_timing_information() {
    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .render(LegacyOptionsBuilder::default().data("a {b: c}").build())
      .unwrap();
    assert!(res.stats.start <= res.stats.end);
    assert_eq!(
      res.stats.duration,
      res.stats.end.duration_since(res.stats.start).unwrap(),
    );
  }

  mod included_files {
    use super::*;

    #[test]
    fn contains_the_root_path_with_a_file_parameter() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("test.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .build(),
        )
        .unwrap();
      assert!(res.stats.included_files.contains(
        &sandbox
          .path()
          .join("test.scss")
          .to_str()
          .unwrap()
          .to_string()
      ));
    }

    #[test]
    fn does_not_contain_the_root_path_with_a_data_parameter() {
      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .render(LegacyOptionsBuilder::default().data("a {b: c}").build())
        .unwrap();
      assert!(res.stats.included_files.is_empty());
    }

    #[test]
    fn contains_imported_paths() {
      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("_other.scss"), "a {b: c}")
        .write(sandbox.path().join("test.scss"), "@import \"other\"");

      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .build(),
        )
        .unwrap();
      assert!(res.stats.included_files.contains(
        &sandbox
          .path()
          .join("_other.scss")
          .to_str()
          .unwrap()
          .to_string()
      ));
    }

    #[test]
    fn only_contains_each_path_once() {
      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("_other.scss"), "a {b: c}")
        .write(sandbox.path().join("test.scss"), "@import \"other\"");

      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .render(
          LegacyOptionsBuilder::default()
            .file(sandbox.path().join("test.scss"))
            .build(),
        )
        .unwrap();
      assert!(
        res
          .stats
          .included_files
          .iter()
          .filter(
            |p| p == &sandbox.path().join("_other.scss").to_str().unwrap()
          )
          .count()
          == 1
      );
    }
  }
}

mod throws_a_legacy_exception {
  use super::*;

  #[test]
  fn for_a_parse_error_in_a_file() {
    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {b: }");

    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
    assert_eq!(
      err.span().unwrap().url.as_ref().unwrap(),
      &sandbox.path().join("test.scss").to_url(),
    );
  }

  #[test]
  fn for_a_parse_error_in_a_string() {
    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .render(LegacyOptionsBuilder::default().data("a {b: }").build())
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
    assert!(err.span().unwrap().url.is_none());
  }

  #[test]
  fn for_a_runtime_error_in_a_file() {
    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("test.scss"), "a {b: 1 % a}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(sandbox.path().join("test.scss"))
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
    assert_eq!(
      err.span().unwrap().url.as_ref().unwrap(),
      &sandbox.path().join("test.scss").to_url(),
    );
  }

  #[test]
  fn for_a_runtime_error_in_a_string() {
    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .render(LegacyOptionsBuilder::default().data("a {b: 1 % a}").build())
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
    assert!(err.span().unwrap().url.is_none());
  }
}
