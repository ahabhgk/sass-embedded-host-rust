#[path = "helpers.rs"]
mod helpers;

use helpers::{exe_path, Sandbox, ToUrl};
use sass_embedded_host_rust::{
  Options, OptionsBuilder, OutputStyle, Sass, StringOptions,
  StringOptionsBuilder, Syntax, Url,
};
use serde_json::json;

mod compile_string {
  use super::*;

  mod success {
    use super::*;

    mod input {
      use super::*;

      #[test]
      fn compiles_scss_by_default() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string("$a: b; c {d: $a}", StringOptions::default())
          .unwrap();
        assert_eq!(res.css, "c {\n  d: b;\n}");
      }

      #[test]
      fn compiles_scss_with_explicit_syntax() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "$a: b; c {d: $a}",
            StringOptionsBuilder::default().syntax(Syntax::Scss).build(),
          )
          .unwrap();
        assert_eq!(res.css, "c {\n  d: b;\n}");
      }

      #[test]
      fn compiles_indented_syntax_with_explicit_syntax() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a\n  b: c",
            StringOptionsBuilder::default()
              .syntax(Syntax::Indented)
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}");
      }

      #[test]
      fn compiles_plain_css_with_explicit_syntax() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a {b: c}",
            StringOptionsBuilder::default().syntax(Syntax::Css).build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}");
      }

      #[test]
      fn does_not_take_its_syntax_from_the_url_s_extension() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a {b: c}",
            StringOptionsBuilder::default()
              .url(Url::parse("file:///foo.sass").unwrap())
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}");
      }
    }

    mod loaded_urls {
      use super::*;

      #[test]
      fn is_empty_with_no_url() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string("a {b: c}", StringOptions::default())
          .unwrap();
        assert!(res.loaded_urls.is_empty());
      }

      #[test]
      fn contains_the_url_if_one_is_passed() {
        let url = Url::parse("file:///foo.scss").unwrap();
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a {b: c}",
            StringOptionsBuilder::default().url(url.clone()).build(),
          )
          .unwrap();
        assert_eq!(res.loaded_urls, vec![url]);
      }

      #[test]
      fn contains_an_immediate_dependency() {
        let sandbox = Sandbox::default();
        let url = sandbox.path().join("input.scss").to_url();
        sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"other\"",
            StringOptionsBuilder::default().url(url.clone()).build(),
          )
          .unwrap();
        assert_eq!(
          res.loaded_urls,
          vec![url, sandbox.path().join("_other.scss").to_url(),]
        );
      }

      #[test]
      fn contains_a_transitive_dependency() {
        let sandbox = Sandbox::default();
        let url = sandbox.path().join("input.scss").to_url();
        sandbox
          .write(sandbox.path().join("_midstream.scss"), "@use \"upstream\"")
          .write(sandbox.path().join("_upstream.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"midstream\"",
            StringOptionsBuilder::default().url(url.clone()).build(),
          )
          .unwrap();
        assert_eq!(
          res.loaded_urls,
          vec![
            url,
            sandbox.path().join("_midstream.scss").to_url(),
            sandbox.path().join("_upstream.scss").to_url(),
          ]
        );
      }

      mod contains_a_dependency_only_once {
        use super::*;

        #[test]
        fn for_at_use() {
          let sandbox = Sandbox::default();
          let url = sandbox.path().join("input.scss").to_url();
          sandbox
            .write(sandbox.path().join("_left.scss"), "@use \"upstream\"")
            .write(sandbox.path().join("_right.scss"), "@use \"upstream\"")
            .write(sandbox.path().join("_upstream.scss"), "a {b: c}");

          let mut sass = Sass::new(exe_path());
          let res = sass
            .compile_string(
              "@use \"left\"; @use \"right\"",
              StringOptionsBuilder::default().url(url.clone()).build(),
            )
            .unwrap();
          assert_eq!(
            res.loaded_urls,
            vec![
              url,
              sandbox.path().join("_left.scss").to_url(),
              sandbox.path().join("_upstream.scss").to_url(),
              sandbox.path().join("_right.scss").to_url(),
            ]
          );
        }

        #[test]
        fn for_at_import() {
          let sandbox = Sandbox::default();
          let url = sandbox.path().join("input.scss").to_url();
          sandbox
            .write(sandbox.path().join("_left.scss"), "@use \"upstream\"")
            .write(sandbox.path().join("_right.scss"), "@use \"upstream\"")
            .write(sandbox.path().join("_upstream.scss"), "a {b: c}");

          let mut sass = Sass::new(exe_path());
          let res = sass
            .compile_string(
              "@import \"left\"; @import \"right\"",
              StringOptionsBuilder::default().url(url.clone()).build(),
            )
            .unwrap();
          assert_eq!(
            res.loaded_urls,
            vec![
              url,
              sandbox.path().join("_left.scss").to_url(),
              sandbox.path().join("_upstream.scss").to_url(),
              sandbox.path().join("_right.scss").to_url(),
            ]
          );
        }
      }
    }

    #[test]
    fn file_url_is_used_to_resolve_relative_loads() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("foo/bar/_other.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile_string(
          "@use \"other\";",
          StringOptionsBuilder::default()
            .url(sandbox.path().join("foo/bar/style.scss").to_url())
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}",);
    }

    mod load_paths {
      use super::*;

      #[test]
      fn is_used_to_resolve_loads() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("foo/bar/_other.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"other\";",
            StringOptionsBuilder::default()
              .load_path(sandbox.path().join("foo/bar"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}",);
      }

      #[test]
      fn resolves_relative_paths() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("foo/bar/_other.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"bar/other\";",
            StringOptionsBuilder::default()
              .load_path(sandbox.path().join("foo"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}",);
      }

      #[test]
      fn resolves_loads_using_later_paths_if_earlier_ones_do_not_match() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("bar/_other.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"other\";",
            StringOptionsBuilder::default()
              .load_path(sandbox.path().join("foo"))
              .load_path(sandbox.path().join("bar"))
              .load_path(sandbox.path().join("baz"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}",);
      }

      #[test]
      fn does_not_take_precedence_over_loads_relative_to_the_url() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("url/_other.scss"), "a {b: url}")
          .write(
            sandbox.path().join("load-path/_other.scss"),
            "a {b: load path}",
          );

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"other\";",
            StringOptionsBuilder::default()
              .load_path(sandbox.path().join("load-path"))
              .url(sandbox.path().join("url/input.scss").to_url())
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: url;\n}",);
      }

      #[test]
      fn uses_earlier_paths_in_preference_to_later_ones() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("earlier/_other.scss"), "a {b: earlier}")
          .write(sandbox.path().join("later/_other.scss"), "a {b: later}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "@use \"other\";",
            StringOptionsBuilder::default()
              .load_path(sandbox.path().join("earlier"))
              .load_path(sandbox.path().join("later"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: earlier;\n}",);
      }
    }

    #[test]
    fn recognizes_the_expanded_output_style() {
      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile_string(
          "a {b: c}",
          StringOptionsBuilder::default()
            .style(OutputStyle::Expanded)
            .build(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}",);
    }

    mod source_map {
      use super::*;

      #[test]
      fn does_not_include_one_by_default() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string("a {b: c}", StringOptions::default())
          .unwrap();
        assert!(res.source_map.is_none());
      }

      #[test]
      fn includes_one_if_source_map_is_true() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a {b: c}",
            StringOptionsBuilder::default().source_map(true).build(),
          )
          .unwrap();
        assert!(res.source_map.is_some());
        let source_map: serde_json::Value =
          serde_json::from_str(&res.source_map.unwrap()).unwrap();
        assert_eq!(source_map["version"], json!(3));
        assert!(source_map["sources"].is_array());
        assert!(source_map["names"].is_array());
        assert!(source_map["mappings"].is_string());
      }

      #[test]
      fn includes_one_with_source_content_if_source_map_include_sources_is_true(
      ) {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a {b: c}",
            StringOptionsBuilder::default()
              .source_map(true)
              .source_map_include_sources(true)
              .build(),
          )
          .unwrap();
        assert!(res.source_map.is_some());
        let source_map: serde_json::Value =
          serde_json::from_str(&res.source_map.unwrap()).unwrap();
        assert!(source_map.get("sourcesContent").is_some());
        assert!(source_map["sourcesContent"].is_array());
        assert!(!source_map["sourcesContent"].as_array().unwrap().is_empty());
      }
    }

    mod charset {
      use super::*;

      #[test]
      fn emits_at_charset_utf_8_or_bom_for_non_ascii_css_by_default() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string("a {b: あ;}", StringOptions::default())
          .unwrap();
        assert_eq!(res.css, "@charset \"UTF-8\";\na {\n  b: あ;\n}");
      }

      #[test]
      fn does_not_emit_at_charset_or_bom_if_charset_is_false() {
        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile_string(
            "a {b: あ;}",
            StringOptionsBuilder::default().charset(false).build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: あ;\n}");
      }
    }
  }

  mod error {
    use super::*;

    #[test]
    fn requires_plain_css_with_explicit_syntax() {
      let mut sass = Sass::new(exe_path());
      let err = sass
        .compile_string(
          "$a: b; c {d: $a}",
          StringOptionsBuilder::default().syntax(Syntax::Css).build(),
        )
        .unwrap_err();
      assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
      assert_eq!(err.span().unwrap().url, String::new());
    }

    #[test]
    fn relative_loads_fail_without_a_url() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let err = sass
        .compile_string("@use \"./other\"", StringOptions::default())
        .unwrap_err();
      assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
      assert_eq!(err.span().unwrap().url, String::new());
    }

    #[test]
    fn relative_loads_fail_with_a_non_file_url() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let err = sass
        .compile_string(
          "@use \"./other\"",
          StringOptionsBuilder::default()
            .url(Url::parse("unknown:style.scss").unwrap())
            .build(),
        )
        .unwrap_err();
      assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
      assert_eq!(err.span().unwrap().url, "unknown:style.scss".to_owned());
    }

    mod includes_source_span_information {
      use super::*;

      #[test]
      fn in_syntax_errors() {
        let sandbox = Sandbox::default();
        let url = sandbox.path().join("foo.scss").to_url();

        let mut sass = Sass::new(exe_path());
        let err = sass
          .compile_string(
            "a {b:",
            StringOptionsBuilder::default().url(url.clone()).build(),
          )
          .unwrap_err();
        assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
        assert_eq!(err.span().unwrap().url, url.to_string());
      }

      #[test]
      fn in_runtime_errors() {
        let sandbox = Sandbox::default();
        let url = sandbox.path().join("foo.scss").to_url();

        let mut sass = Sass::new(exe_path());
        let err = sass
          .compile_string(
            "@error \"oh no\"",
            StringOptionsBuilder::default().url(url.clone()).build(),
          )
          .unwrap_err();
        assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
        assert_eq!(err.span().unwrap().url, url.to_string());
      }
    }
  }
}

mod compile {
  use super::*;

  mod success {
    use super::*;

    #[test]
    fn compiles_scss_for_a_scss_file() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("input.scss"), "$a: b; c {d: $a}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile(sandbox.path().join("input.scss"), Options::default())
        .unwrap();
      assert_eq!(res.css, "c {\n  d: b;\n}");
    }

    #[test]
    fn compiles_scss_for_a_file_with_an_unknown_extension() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("input.asdf"), "$a: b; c {d: $a}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile(sandbox.path().join("input.asdf"), Options::default())
        .unwrap();
      assert_eq!(res.css, "c {\n  d: b;\n}");
    }

    #[test]
    fn compiles_indented_syntax_for_a_sass_file() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("input.sass"), "a\n  b: c");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile(sandbox.path().join("input.sass"), Options::default())
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}");
    }

    #[test]
    fn compiles_plain_css_for_a_css_file() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("input.css"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile(sandbox.path().join("input.css"), Options::default())
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}");
    }

    mod loaded_urls {
      use super::*;

      #[test]
      fn includes_a_relative_path_s_url() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("input.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile(sandbox.path().join("input.scss"), Options::default())
          .unwrap();
        assert_eq!(
          res.loaded_urls,
          vec![sandbox.path().join("input.scss").to_url()]
        );
      }

      #[test]
      fn includes_an_absolute_path_s_url() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("input.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile(sandbox.path().join("input.scss"), Options::default())
          .unwrap();
        assert_eq!(
          res.loaded_urls,
          vec![sandbox.path().join("input.scss").to_url()]
        );
      }

      #[test]
      fn contains_a_dependency() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("input.scss"), "@use \"other\"")
          .write(sandbox.path().join("_other.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile(sandbox.path().join("input.scss"), Options::default())
          .unwrap();
        assert_eq!(
          res.loaded_urls,
          vec![
            sandbox.path().join("input.scss").to_url(),
            sandbox.path().join("_other.scss").to_url(),
          ]
        );
      }
    }

    #[test]
    fn the_path_is_used_to_resolve_relative_loads() {
      let sandbox = Sandbox::default();
      sandbox
        .write(sandbox.path().join("foo/bar/input.scss"), "@use \"other\"")
        .write(sandbox.path().join("foo/bar/_other.scss"), "a {b: c}");

      let mut sass = Sass::new(exe_path());
      let res = sass
        .compile(
          sandbox.path().join("foo/bar/input.scss"),
          Options::default(),
        )
        .unwrap();
      assert_eq!(res.css, "a {\n  b: c;\n}");
    }

    mod load_paths {
      use super::*;

      #[test]
      fn is_used_to_resolve_loads() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("input.scss"), "@use \"other\"")
          .write(sandbox.path().join("foo/bar/_other.scss"), "a {b: c}");

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile(
            sandbox.path().join("input.scss"),
            OptionsBuilder::default()
              .load_path(sandbox.path().join("foo/bar"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: c;\n}");
      }

      #[test]
      fn does_not_take_precedence_over_loads_relative_to_the_entrypoint() {
        let sandbox = Sandbox::default();
        sandbox
          .write(sandbox.path().join("url/input.scss"), "@use \"other\"")
          .write(sandbox.path().join("url/_other.scss"), "a {b: url}")
          .write(
            sandbox.path().join("load-path/_other.scss"),
            "a {b: load path}",
          );

        let mut sass = Sass::new(exe_path());
        let res = sass
          .compile(
            sandbox.path().join("url/input.scss"),
            OptionsBuilder::default()
              .load_path(sandbox.path().join("load-path"))
              .build(),
          )
          .unwrap();
        assert_eq!(res.css, "a {\n  b: url;\n}");
      }
    }
  }

  mod error {
    use super::*;

    #[test]
    fn requires_plain_css_for_a_css_file() {
      let sandbox = Sandbox::default();
      sandbox.write(sandbox.path().join("input.css"), "$a: b; c {d: $a}");

      let mut sass = Sass::new(exe_path());
      let err = sass
        .compile(sandbox.path().join("input.css"), Options::default())
        .unwrap_err();
      assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
      assert_eq!(
        err.span().unwrap().url,
        sandbox.path().join("input.css").to_url().to_string()
      );
    }

    mod includes_the_path_s_url {
      use super::*;

      #[test]
      fn in_syntax_errors() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("input.css"), "a {b:");

        let mut sass = Sass::new(exe_path());
        let err = sass
          .compile(sandbox.path().join("input.css"), Options::default())
          .unwrap_err();
        assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
        assert_eq!(
          err.span().unwrap().url,
          sandbox.path().join("input.css").to_url().to_string()
        );
      }

      #[test]
      fn in_runtime_errors() {
        let sandbox = Sandbox::default();
        sandbox.write(sandbox.path().join("input.css"), "@error \"oh no\"");

        let mut sass = Sass::new(exe_path());
        let err = sass
          .compile(sandbox.path().join("input.css"), Options::default())
          .unwrap_err();
        assert_eq!(err.span().unwrap().start.as_ref().unwrap().line, 0);
        assert_eq!(
          err.span().unwrap().url,
          sandbox.path().join("input.css").to_url().to_string()
        );
      }
    }
  }
}
