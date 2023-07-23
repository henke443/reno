extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::Context;

use anyhow::__private::kind::TraitKind;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use thiserror::Error;

use std::str;

use itertools::Itertools;
use rayon::prelude::*;

use regex::Regex;

use std::fs::File;

use clap::Parser;

use std::fs;
use std::fs::*;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;

use std::io::{BufRead, BufReader, Read};

use encoding_rs_io::DecodeReaderBytesBuilder;

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
    str_search: Option<String>,
    str_replace: &str,
    b_dry: bool,
) -> Result<Vec<NameReplacementInfo>> {
    let re = if str_search.is_some() {
        Regex::new(&str_search.unwrap())?
    } else {
        Regex::new(".*")?
    };

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