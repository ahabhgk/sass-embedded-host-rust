mod api;
mod importer;

use std::{sync::Arc, time::SystemTime};

pub use api::*;
pub use importer::*;

use crate::{Embedded, Importer, Options, Result, SassImporter, StringOptions};

impl Embedded {
  pub fn render(&mut self, options: LegacyOptions) -> Result<LegacyResult> {
    let start = SystemTime::now();
    let entry = options
      .file
      .clone()
      .map(|file| file.to_str().unwrap().to_string())
      .unwrap_or_else(|| "stdin".to_string());
    let mut options = options.adjust_options();
    let result = if let Some(data) = options.data.clone() {
      let this = LegacyPluginThis::new(&options);
      let (importers, input_importer) =
        if let Some(importers) = options.importers.take() {
          let wrapper = LegacyImporterWrapper::new(
            this,
            importers,
            options.include_paths.clone(),
            &entry,
          );
          let importers = vec![SassImporter::Importer(Box::new(Arc::clone(
            &wrapper,
          ))
            as Box<dyn Importer>)];
          let input_importer = Some(SassImporter::Importer(
            Box::new(wrapper) as Box<dyn Importer>
          ));
          (importers, input_importer)
        } else {
          (Vec::new(), None)
        };
      let mut options = StringOptions::from(options);
      options.common.importers = importers;
      options.input_importer = input_importer;
      self.compile_string(data, options)
    } else if let Some(file) = options.file.clone() {
      let this = LegacyPluginThis::new(&options);
      let importers = options
        .importers
        .take()
        .map(|importers| {
          let wrapper = LegacyImporterWrapper::new(
            this,
            importers,
            options.include_paths.clone(),
            &entry,
          );
          let importers = vec![SassImporter::Importer(Box::new(Arc::clone(
            &wrapper,
          ))
            as Box<dyn Importer>)];
          importers
        })
        .unwrap_or_default();
      let mut options = Options::from(options);
      options.importers = importers;
      self.compile(file, options)
    } else {
      panic!("Either options.data or options.file must be set.");
    }?;
    Ok(LegacyResult::new(entry, start, result))
  }
}
