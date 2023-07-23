extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::Context;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use std::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoNamesError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Error renaming file: {0}")]
    RenameError(String, #[source] std::io::Error),
    #[error("Invalid filename: {0}")]
    InvalidFilename(Box<Path>),
}

#[derive(Debug)]
pub struct NameReplacementInfo {
    pub did_change: bool,
    pub path: PathBuf,
    pub old_name: String,
    pub new_name: String,
}

pub fn do_names(
    source_path: &Path,
    str_search: &str,
    str_replace: &str,
    b_dry: bool,
) -> Result<Vec<NameReplacementInfo>> {
    let re = Regex::new(&str_search)?;

    let old_path = source_path;
    let old_name = old_path
        .file_name()
        .context(DoNamesError::InvalidFilename(Box::from(old_path)))?
        .clone()
        .to_str()
        .context(DoNamesError::InvalidFilename(Box::from(old_path)))?;

    let new_name = re.replace_all(&old_name, str_replace);
    let new_path = old_path.with_file_name(PathBuf::from(new_name.clone().into_owned()));

    let mut name_replacements_info = vec![];

    if new_name != old_name {
        let mut replacement_was_ok = false;

        if !b_dry {
            fs::rename(&old_path, &new_path)
                .or_else(|err| {
                    Err(DoNamesError::RenameError(
                        new_name.clone().into_owned(),
                        err,
                    ))
                })
                .and_then(|_| {
                    replacement_was_ok = true;
                    Ok(())
                })?;
        }

        name_replacements_info.push(NameReplacementInfo {
            did_change: !b_dry && replacement_was_ok,
            path: old_path.to_path_buf(),
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        });
    }

    Ok(name_replacements_info)
}
