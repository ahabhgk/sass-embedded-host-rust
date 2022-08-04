#[path = "./helpers.rs"]
mod helpers;

use helpers::{exe_path, Sandbox, ToUrl};
#[cfg(feature = "legacy")]
use sass_embedded_host_rust::{
  legacy::{
    LegacyImporter, LegacyImporterResult, LegacyImporterThis, LegacyOptions,
    LegacyOptionsBuilder,
  },
  Options, OptionsBuilder, OutputStyle, Result, Sass, StringOptions,
  StringOptionsBuilder, Syntax, Url,
};

#[cfg(feature = "legacy")]
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
        return Ok(Some(LegacyImporterResult::Contents(
          "@import \"bar\"".to_owned(),
        )));
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
        return Ok(Some(LegacyImporterResult::Contents(
          "@import \"baz\"".to_owned(),
        )));
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
        return Ok(Some(LegacyImporterResult::Contents("a {b: c}".to_owned())));
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

#[cfg(feature = "legacy")]
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
        return Ok(Some(LegacyImporterResult::Contents(String::new())));
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
