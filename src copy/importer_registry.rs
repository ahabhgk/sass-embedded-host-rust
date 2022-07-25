use std::collections::HashMap;

use url::Url;

use crate::{
  api::{FileImporter, Importer, ImporterOptions, Result, SassImporter},
  error::Error,
  pb::{
    inbound_message::{
      canonicalize_response, compile_request, file_import_response,
      import_response, CanonicalizeResponse, FileImportResponse,
      ImportResponse,
    },
    outbound_message::{CanonicalizeRequest, FileImportRequest, ImportRequest},
  },
};

/// A registry of importers defined in the host that can be invoked by the
/// compiler.
pub struct ImporterRegistry {
  /// Protocol buffer representations of the registered importers.
  importers: Vec<compile_request::Importer>,
  /// The next ID to use for an importer.
  id: u32,
  /// A map from importer IDs to their corresponding importers.
  importers_by_id: HashMap<u32, Box<dyn Importer>>,
  /// A map from file importer IDs to their corresponding importers.
  file_importers_by_id: HashMap<u32, Box<dyn FileImporter>>,
}

impl ImporterRegistry {
  pub fn new(
    importers: Option<Vec<SassImporter>>,
    load_paths: Option<Vec<String>>,
  ) -> Self {
    let mut this = Self {
      importers: Vec::new(),
      id: 0,
      importers_by_id: HashMap::new(),
      file_importers_by_id: HashMap::new(),
    };
    let mut importers: Vec<compile_request::Importer> = importers
      .unwrap_or_default()
      .into_iter()
      .map(|importer| this.register(importer))
      .collect();
    importers.extend(load_paths.unwrap_or_default().into_iter().map(|p| {
      let i = compile_request::importer::Importer::Path(p);
      compile_request::Importer::new(i)
    }));
    this.importers = importers;
    this
  }

  /// Get all protofied importers.
  pub fn importers(&self) -> Vec<compile_request::Importer> {
    self.importers.clone()
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
  pub async fn canonicalize(
    &self,
    request: &CanonicalizeRequest,
  ) -> Result<CanonicalizeResponse> {
    let importer =
      self
        .importers_by_id
        .get(&request.importer_id)
        .ok_or_else(|| {
          Error::Compile("Unknown CanonicalizeRequest.importer_id".to_string())
        })?;
    match importer
      .canonicalize(
        &request.url,
        &ImporterOptions {
          from_import: request.from_import,
        },
      )
      .await
    {
      Ok(url) => {
        let mut proto = CanonicalizeResponse::default();
        if let Some(url) = url {
          proto.result =
            Some(canonicalize_response::Result::Url(url.to_string()));
        };
        Ok(proto)
      }
      Err(e) => Ok(CanonicalizeResponse {
        result: Some(canonicalize_response::Result::Error(e.to_string())),
        ..Default::default()
      }),
    }
  }

  /// Handles an import request.
  pub async fn import(
    &self,
    request: &ImportRequest,
  ) -> Result<ImportResponse> {
    let importer =
      self
        .importers_by_id
        .get(&request.importer_id)
        .ok_or_else(|| {
          Error::Compile("Unknown ImportRequest.importer_id".to_string())
        })?;
    match importer.load(&Url::parse(&request.url).unwrap()).await {
      Ok(result) => {
        let mut proto = ImportResponse::default();
        if let Some(result) = result {
          let mut success = import_response::ImportSuccess {
            contents: result.contents,
            ..Default::default()
          };
          success.set_syntax(result.syntax);
          if let Some(source_map_url) = result.source_map_url {
            success.source_map_url = source_map_url;
          }
          proto.result = Some(import_response::Result::Success(success));
        };
        Ok(proto)
      }
      Err(e) => Ok(ImportResponse {
        result: Some(import_response::Result::Error(e.to_string())),
        ..Default::default()
      }),
    }
  }

  /// Handles a file import request.
  pub async fn file_import(
    &self,
    request: &FileImportRequest,
  ) -> Result<FileImportResponse> {
    let importer = self
      .file_importers_by_id
      .get(&request.importer_id)
      .ok_or_else(|| {
        Error::Compile("Unknown FileImportRequest.importer_id".to_string())
      })?;
    match importer
      .find_file_url(
        &request.url,
        &ImporterOptions {
          from_import: request.from_import,
        },
      )
      .await
    {
      Ok(url) => {
        let mut proto = FileImportResponse::default();
        if let Some(url) = url {
          if url.scheme() != "file" {
            return Err(Error::Host(format!(
              "FileImporter {:?} returned non-file: URL {} for URL {}.",
              importer, url, request.url
            )));
          }
          proto.result =
            Some(file_import_response::Result::FileUrl(url.to_string()));
        };
        Ok(proto)
      }
      Err(e) => Ok(FileImportResponse {
        result: Some(file_import_response::Result::Error(e.to_string())),
        ..Default::default()
      }),
    }
  }
}
