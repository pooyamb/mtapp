use std::{
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use serde::Deserialize;

use crate::BoxError;

#[derive(Debug, Deserialize)]
pub(crate) struct MigrationData {
    pub(crate) dependencies: Vec<String>,
    pub(crate) description: String,

    #[serde(skip_deserializing)]
    pub(crate) name: String,

    #[serde(skip_deserializing)]
    pub(crate) up: String,
    #[serde(skip_deserializing)]
    pub(crate) down: String,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Invalid .meta.json format in {0}")]
    MetaInvalid(PathBuf),
    #[error(".meta.json not found in {0}")]
    MetaNotFound(PathBuf),
    #[error("up.sql not found in {0}")]
    UpSqlNotFound(PathBuf),
    #[error("up.sql has invalid format in {0}")]
    UpSqlInvalid(PathBuf),
    #[error("down.sql not found in {0}")]
    DownSqlNotFound(PathBuf),
    #[error("down.sql has invalid format in {0}")]
    DownSqlInvalid(PathBuf),
}

pub(crate) fn read_migrations(path: PathBuf) -> Result<Vec<MigrationData>, BoxError> {
    let mut migrations = Vec::new();
    for folder in fs::read_dir(path)? {
        if let Ok(folder) = folder {
            if !folder.file_type()?.is_dir() {
                continue;
            }
            let name = folder.file_name();

            // Read the meta json
            let mut json_path = folder.path();
            json_path.push(".meta.json");
            let meta_data =
                fs::read(&json_path).map_err(|_| Error::MetaNotFound(json_path.clone()))?;
            let mut migration: MigrationData =
                serde_json::from_slice(&meta_data).map_err(|_| Error::MetaInvalid(json_path))?;

            let mut up_path = folder.path();
            up_path.push("up.sql");
            let up = fs::read(&up_path).map_err(|_| Error::UpSqlNotFound(up_path.clone()))?;
            migration.up = String::from_utf8(up).map_err(|_| Error::UpSqlInvalid(up_path))?;

            let mut down_path = folder.path();
            down_path.push("down.sql");
            let down =
                fs::read(&down_path).map_err(|_| Error::DownSqlNotFound(down_path.clone()))?;
            migration.down =
                String::from_utf8(down).map_err(|_| Error::DownSqlInvalid(down_path))?;

            migration.name = String::from(name.to_str().expect("Name is not valid utf-8"));

            migrations.push(migration)
        }
    }
    Ok(migrations)
}

/// Straight from sqlx-macros
pub(crate) fn resolve_path(path: impl AsRef<Path>, err_span: Span) -> syn::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        return Err(syn::Error::new(
            err_span,
            "absolute paths will only work on the current machine",
        ));
    }

    // requires `proc_macro::SourceFile::path()` to be stable
    // https://github.com/rust-lang/rust/issues/54725
    if path.is_relative()
        && !path
            .parent()
            .map_or(false, |parent| !parent.as_os_str().is_empty())
    {
        return Err(syn::Error::new(
            err_span,
            "paths relative to the current file's directory are not currently supported",
        ));
    }

    let base_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(
            err_span,
            "CARGO_MANIFEST_DIR is not set; please use Cargo to build",
        )
    })?;
    let base_dir_path = Path::new(&base_dir);

    Ok(base_dir_path.join(path))
}
