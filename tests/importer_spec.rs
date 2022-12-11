#[path = "helpers.rs"]
mod helpers;

use helpers::{exe_path, Sandbox, ToUrl};
use parking_lot::Mutex;
use sass_embedded::{
  Exception, FileImporter, Importer, ImporterOptions, ImporterResult,
  OptionsBuilder, Result, Sass, StringOptions, StringOptionsBuilder, Syntax,
  Url,
};
use serde_json::{json, Value};

#[test]
fn uses_an_importer_to_resolve_an_at_import() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      url: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
    }

    fn load(&self, canonical_url: &Url) -> Result<Option<ImporterResult>> {
      let color = canonical_url.path();
      Ok(Some(ImporterResult {
        contents: format!(".{color} {{color: {color}}}"),
        syntax: Syntax::Scss,
        source_map_url: None,
      }))
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      "@import \"orange\";",
      StringOptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap();
  assert_eq!(res.css, ".orange {\n  color: orange;\n}");
}

#[test]
fn passes_the_canonicalized_url_to_the_importer() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse("u:blue").unwrap()))
    }

    fn load(&self, canonical_url: &Url) -> Result<Option<ImporterResult>> {
      let color = canonical_url.path();
      Ok(Some(ImporterResult {
        contents: format!(".{color} {{color: {color}}}"),
        syntax: Syntax::Scss,
        source_map_url: None,
      }))
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      "@import \"orange\";",
      StringOptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap();
  assert_eq!(res.css, ".blue {\n  color: blue;\n}");
}

#[test]
fn only_invokes_the_importer_once_for_a_given_canonicalization() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse("u:blue").unwrap()))
    }

    fn load(&self, canonical_url: &Url) -> Result<Option<ImporterResult>> {
      let color = canonical_url.path();
      Ok(Some(ImporterResult {
        contents: format!(".{color} {{color: {color}}}"),
        syntax: Syntax::Scss,
        source_map_url: None,
      }))
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      r#"
      @import "orange";
      @import "orange";
      "#,
      StringOptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap();
  assert_eq!(
    res.css,
    ".blue {\n  color: blue;\n}\n\n.blue {\n  color: blue;\n}"
  );
}

mod the_imported_url {
  use super::*;

  #[test]
  fn is_not_changed_if_it_is_root_relative() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        url: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert_eq!(url, "/orange");
        Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: "a {b: c}".to_string(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"/orange\";",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn is_converted_to_a_file_url_if_it_is_an_absolute_windows_path() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        url: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert_eq!(url, "file:///C:/orange");
        Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: "a {b: c}".to_string(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"C:/orange\";",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }
}

#[test]
fn uses_an_importer_is_source_map_url() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      url: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
    }

    fn load(&self, canonical_url: &Url) -> Result<Option<ImporterResult>> {
      let color = canonical_url.path();
      Ok(Some(ImporterResult {
        contents: format!(".{color} {{color: {color}}}"),
        syntax: Syntax::Scss,
        source_map_url: Some(Url::parse("u:blue").unwrap()),
      }))
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      "@import \"orange\";",
      StringOptionsBuilder::default()
        .importer(MyImporter)
        .source_map(true)
        .build(),
    )
    .unwrap();
  let source_map: Value =
    serde_json::from_str(&res.source_map.unwrap()).unwrap();
  let sources = source_map["sources"].as_array().unwrap();
  assert!(sources.contains(&json!("u:blue")));
}

#[test]
fn wraps_an_error_in_canonicalize() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Err(Exception::new("this import is bad actually").into())
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      panic!("load() should not be called")
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let err = sass
    .compile_string(
      "@import \"orange\";",
      StringOptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap_err();
  assert!(err.message().contains("this import is bad actually"));
  assert_eq!(err.span().unwrap().start.line, 0);
}

#[test]
fn wraps_an_error_in_load() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      url: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      Err(Exception::new("this import is bad actually").into())
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let err = sass
    .compile_string(
      "@import \"orange\";",
      StringOptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap_err();
  assert!(err.message().contains("this import is bad actually"));
  assert_eq!(err.span().unwrap().start.line, 0);
}

#[test]
fn avoids_importer_when_canonicalize_returns_nil() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(None)
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      Err(Exception::new("this import is bad actually").into())
    }
  }

  let sandbox = Sandbox::default();
  sandbox.write(sandbox.path().join("dir/_other.scss"), "a {from: dir}");

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      "@import \"other\";",
      StringOptionsBuilder::default()
        .importer(MyImporter)
        .load_path(sandbox.path().join("dir"))
        .build(),
    )
    .unwrap();
  assert_eq!(res.css, "a {\n  from: dir;\n}");
}

#[test]
fn fails_to_import_when_load_returns_nil() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      url: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      Ok(None)
    }
  }

  let sandbox = Sandbox::default();
  sandbox.write(sandbox.path().join("dir/_other.scss"), "a {from: dir}");

  let mut sass = Sass::new(exe_path()).unwrap();
  let err = sass
    .compile_string(
      "@import \"other\";",
      StringOptionsBuilder::default()
        .importer(MyImporter)
        .load_path(sandbox.path().join("dir"))
        .build(),
    )
    .unwrap_err();
  assert!(err.span().unwrap().start.line == 0);
}

#[test]
fn prefers_a_relative_file_load_to_an_importer() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      panic!("canonicalize() should not be called");
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      panic!("load() should not be called");
    }
  }

  let sandbox = Sandbox::default();
  sandbox
    .write(sandbox.path().join("input.scss"), "@import \"other\"")
    .write(sandbox.path().join("_other.scss"), "a {from: relative}");

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile(
      sandbox.path().join("input.scss"),
      OptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap();
  assert_eq!(res.css, "a {\n  from: relative;\n}");
}

#[test]
fn prefers_a_relative_importer_load_to_an_importer() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      panic!("canonicalize() should not be called");
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      panic!("load() should not be called");
    }
  }
  #[derive(Debug, Default)]
  struct MyInputImporter;

  impl Importer for MyInputImporter {
    fn canonicalize(
      &self,
      url: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      Ok(Some(Url::parse(url).unwrap()))
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      Ok(Some(ImporterResult {
        contents: "a {from: relative}".to_string(),
        syntax: Syntax::Scss,
        source_map_url: None,
      }))
    }
  }

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      "@import \"other\";",
      StringOptionsBuilder::default()
        .importer(MyImporter)
        .url(Url::parse("o:style.scss").unwrap())
        .input_importer(MyInputImporter)
        .build(),
    )
    .unwrap();
  assert_eq!(res.css, "a {\n  from: relative;\n}");
}

#[test]
fn prefers_an_importer_to_a_load_path() {
  #[derive(Debug, Default)]
  struct MyImporter;

  impl Importer for MyImporter {
    fn canonicalize(
      &self,
      _: &str,
      _: &ImporterOptions,
    ) -> Result<Option<Url>> {
      panic!("canonicalize() should not be called");
    }

    fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
      panic!("load() should not be called");
    }
  }

  let sandbox = Sandbox::default();
  sandbox
    .write(sandbox.path().join("input.scss"), "@import \"other\"")
    .write(sandbox.path().join("_other.scss"), "a {from: relative}");

  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile(
      sandbox.path().join("input.scss"),
      OptionsBuilder::default().importer(MyImporter).build(),
    )
    .unwrap();
  assert_eq!(res.css, "a {\n  from: relative;\n}");
}

mod with_syntax {
  use super::*;

  #[test]
  fn scss_parses_it_as_scss() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(Url::parse("u:other").unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: "$a: value; b {c: $a}".to_string(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
    assert_eq!(res.css, "b {\n  c: value;\n}");
  }

  #[test]
  fn indented_parses_it_as_the_indented_syntax() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(Url::parse("u:other").unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: "$a: value\nb\n  c: $a".to_string(),
          syntax: Syntax::Indented,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
    assert_eq!(res.css, "b {\n  c: value;\n}");
  }

  #[test]
  fn css_allows_plain_css() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(Url::parse("u:other").unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: "a {b: c}".to_string(),
          syntax: Syntax::Css,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn css_rejects_scss() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(Url::parse("u:other").unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: "$a: value\nb\n  c: $a".to_string(),
          syntax: Syntax::Css,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
  }
}

mod from_import_is {
  use super::*;

  #[test]
  fn true_from_an_at_import() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        url: &str,
        options: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert!(options.from_import);
        Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: String::new(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let _ = sass
      .compile_string(
        "@import \"foo\"",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
  }

  #[test]
  fn false_from_an_at_use() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        url: &str,
        options: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert!(!options.from_import);
        Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: String::new(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let _ = sass
      .compile_string(
        "@use \"foo\"",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
  }

  #[test]
  fn false_from_an_at_forward() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        url: &str,
        options: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert!(!options.from_import);
        Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: String::new(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let _ = sass
      .compile_string(
        "@forward \"foo\"",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
  }

  #[test]
  fn false_from_meta_load_css() {
    #[derive(Debug, Default)]
    struct MyImporter;

    impl Importer for MyImporter {
      fn canonicalize(
        &self,
        url: &str,
        options: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert!(!options.from_import);
        Ok(Some(Url::parse(&format!("u:{url}")).unwrap()))
      }

      fn load(&self, _: &Url) -> Result<Option<ImporterResult>> {
        Ok(Some(ImporterResult {
          contents: String::new(),
          syntax: Syntax::Scss,
          source_map_url: None,
        }))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let _ = sass
      .compile_string(
        "@use \"sass:meta\"; @include meta.load-css(\"\")",
        StringOptionsBuilder::default().importer(MyImporter).build(),
      )
      .unwrap();
  }
}

mod file_importer {
  use super::*;

  #[test]
  fn loads_a_fully_canonicalized_url() {
    #[derive(Debug, Default)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("_other.scss").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn resolves_a_non_canonicalized_url() {
    #[derive(Debug, Default)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("other/_index.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn avoids_importer_when_it_returns_nil() {
    #[derive(Debug, Default)]
    struct MyFileImporter;

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(None)
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.scss"), "a {from: dir}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .load_path(sandbox.path())
          .file_importer(MyFileImporter)
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  from: dir;\n}");
  }

  #[test]
  fn avoids_importer_when_it_returns_an_unresolvable_url() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("nonexistent/other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .load_path(sandbox.path())
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn passes_an_absolute_non_file_url_to_the_importer() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        url: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert_eq!(url, "u:other");
        Ok(Some(self.sandbox.path().join("dir/other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("dir/_other.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"u:other\";",
        StringOptionsBuilder::default()
          .load_path(sandbox.path())
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn does_not_pass_an_absolute_file_url_to_the_importer() {
    #[derive(Debug)]
    struct MyFileImporter;

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        panic!("find_file_url() should not be called")
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        &format!("@import \"{}\";", sandbox.path().join("other").to_url()),
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter)
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn does_not_pass_relative_loads_to_the_importer() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
      count: Mutex<usize>,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        let mut count = self.count.lock();
        if *count > 0 {
          panic!("find_file_url() should only be called once");
        }
        *count += 1;
        Ok(Some(self.sandbox.path().join("upstream").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox
      .write(
        sandbox.path().join("_midstream.scss"),
        "@import \"upstream\"",
      )
      .write(sandbox.path().join("_upstream.scss"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"midstream\";",
        StringOptionsBuilder::default()
          .load_path(sandbox.path())
          .file_importer(MyFileImporter {
            sandbox,
            count: Mutex::new(0),
          })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn wraps_an_error() {
    #[derive(Debug)]
    struct MyFileImporter;

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Err(Exception::new("this import is bad actually").into())
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .compile_string("@import \"other\";", StringOptions::default())
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
  }

  #[test]
  fn rejects_a_non_file_url() {
    #[derive(Debug)]
    struct MyFileImporter;

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(Url::parse("u:other.scss").unwrap()))
      }
    }

    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter)
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
  }

  #[test]
  fn when_the_resolved_file_has_extension_scss_parses_it_as_scss() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.scss"), "$a: value; b {c: $a}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "b {\n  c: value;\n}");
  }

  #[test]
  fn when_the_resolved_file_has_extension_sass_parses_it_as_the_indented_syntax(
  ) {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.sass"), "$a: value\nb\n  c: $a");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "b {\n  c: value;\n}");
  }

  #[test]
  fn when_the_resolved_file_has_extension_css_allows_plain_css() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.css"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let res = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
    assert_eq!(res.css, "a {\n  b: c;\n}");
  }

  #[test]
  fn when_the_resolved_file_has_extension_css_rejects_scss() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        _: &ImporterOptions,
      ) -> Result<Option<Url>> {
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.css"), "$a: value; b {c: $a}");
    let url = sandbox.path().join("_other.css").to_url();

    let mut sass = Sass::new(exe_path()).unwrap();
    let err = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap_err();
    assert_eq!(err.span().unwrap().start.line, 0);
    assert_eq!(err.span().unwrap().url.as_ref().unwrap(), &url);
  }

  #[test]
  fn from_import_is_true_from_an_at_import() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        options: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert!(options.from_import);
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.css"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let _ = sass
      .compile_string(
        "@import \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
  }

  #[test]
  fn from_import_is_false_from_an_at_use() {
    #[derive(Debug)]
    struct MyFileImporter {
      sandbox: Sandbox,
    }

    impl FileImporter for MyFileImporter {
      fn find_file_url(
        &self,
        _: &str,
        options: &ImporterOptions,
      ) -> Result<Option<Url>> {
        assert!(!options.from_import);
        Ok(Some(self.sandbox.path().join("other").to_url()))
      }
    }

    let sandbox = Sandbox::default();
    sandbox.write(sandbox.path().join("_other.css"), "a {b: c}");

    let mut sass = Sass::new(exe_path()).unwrap();
    let _ = sass
      .compile_string(
        "@use \"other\";",
        StringOptionsBuilder::default()
          .file_importer(MyFileImporter { sandbox })
          .build(),
      )
      .unwrap();
  }
}
