#![cfg(feature = "legacy")]

#[path = "./helpers.rs"]
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
