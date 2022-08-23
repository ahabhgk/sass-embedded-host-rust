use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::{
  api::{FileImporter, Importer, ImporterOptions, SassImporter},
  protocol::{
    self,
    inbound_message::{
      canonicalize_response, compile_request, file_import_response,
      import_response::{self, ImportSuccess},
      CanonicalizeResponse, FileImportResponse, ImportResponse,
    },
    outbound_message::{CanonicalizeRequest, FileImportRequest, ImportRequest},
  },
  Url,
};

/// A registry of importers defined in the host that can be invoked by the
/// compiler.
#[derive(Debug, Default)]
pub struct ImporterRegistry {
  /// The next ID to use for an importer.
  id: u32,
  /// A map from importer IDs to their corresponding importers.
  importers_by_id: FxHashMap<u32, Box<dyn Importer>>,
  /// A map from file importer IDs to their corresponding importers.
  file_importers_by_id: FxHashMap<u32, Box<dyn FileImporter>>,
}

impl ImporterRegistry {
  pub fn register_all(
    &mut self,
    importers: Vec<SassImporter>,
    load_paths: Vec<PathBuf>,
  ) -> impl Iterator<Item = compile_request::Importer> + '_ {
    let load_paths: Vec<_> = self.register_load_paths(load_paths).collect();
    self.register_importers(importers).chain(load_paths)
  }

  fn register_importers(
    &mut self,
    importers: Vec<SassImporter>,
  ) -> impl Iterator<Item = compile_request::Importer> + '_ {
    importers
      .into_iter()
      .map(|importer| self.register(importer))
  }

  fn register_load_paths(
    &self,
    load_paths: Vec<PathBuf>,
  ) -> impl Iterator<Item = compile_request::Importer> + '_ {
    load_paths.into_iter().map(|p| {
      let i = compile_request::importer::Importer::Path(
        p.to_str().unwrap().to_string(),
      );
      compile_request::Importer { importer: Some(i) }
    })
  }

  /// Converts an importer to a proto.
  pub fn register(
    &mut self,
    importer: SassImporter,
  ) -> compile_request::Importer {
    let i = match importer {
      SassImporter::Importer(i) => {
        self.importers_by_id.insert(self.id, i);
        compile_request::importer::Importer::ImporterId(self.id)
      }
      SassImporter::FileImporter(i) => {
        self.file_importers_by_id.insert(self.id, i);
        compile_request::importer::Importer::FileImporterId(self.id)
      }
    };
    self.id += 1;
    compile_request::Importer { importer: Some(i) }
  }

  /// Handles a canonicalization request.
  pub fn canonicalize(
    &self,
    request: &CanonicalizeRequest,
  ) -> CanonicalizeResponse {
    let importer = self.importers_by_id.get(&request.importer_id).unwrap();
    match importer.canonicalize(
      &request.url,
      &ImporterOptions {
        from_import: request.from_import,
      },
    ) {
      Ok(url) => CanonicalizeResponse {
        id: request.id,
        result: url
          .map(|url| canonicalize_response::Result::Url(url.to_string())),
      },
      Err(e) => CanonicalizeResponse {
        id: request.id,
        result: Some(canonicalize_response::Result::Error(e.to_string())),
      },
    }
  }

  /// Handles an import request.
  pub fn import(&self, request: &ImportRequest) -> ImportResponse {
    let importer = self.importers_by_id.get(&request.importer_id).unwrap();
    match importer.load(&Url::parse(&request.url).unwrap()) {
      Ok(result) => ImportResponse {
        id: request.id,
        result: if let Some(result) = result {
          Some(import_response::Result::Success(ImportSuccess {
            contents: result.contents,
            syntax: protocol::Syntax::from(result.syntax) as i32,
            source_map_url: result
              .source_map_url
              .map(|url| url.to_string())
              .unwrap_or_default(),
          }))
        } else {
          None
        },
      },
      Err(e) => ImportResponse {
        id: request.id,
        result: Some(import_response::Result::Error(e.to_string())),
      },
    }
  }

  /// Handles a file import request.
  pub fn file_import(&self, request: &FileImportRequest) -> FileImportResponse {
    let importer = self.file_importers_by_id.get(&request.importer_id).unwrap();
    match importer.find_file_url(
      &request.url,
      &ImporterOptions {
        from_import: request.from_import,
      },
    ) {
      Ok(url) => FileImportResponse {
        id: request.id,
        result: url.map(|url| {
          if url.scheme() != "file" {
            file_import_response::Result::Error(format!(
              "FileImporter {:?} returned non-file: URL {} for URL {}.",
              importer, url, request.url
            ))
          } else {
            file_import_response::Result::FileUrl(url.to_string())
          }
        }),
      },
      Err(e) => FileImportResponse {
        id: request.id,
        result: Some(file_import_response::Result::Error(e.to_string())),
      },
    }
  }
}
