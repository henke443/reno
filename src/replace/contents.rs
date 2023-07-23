extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::ensure;
use anyhow::Result;
use encoding_rs_io::DecodeReaderBytesBuilder;
use itertools::Itertools;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::str;
use std::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoContentError {
    #[error("Empty file\n")]
    EmptyFile,
    #[error("File \"{0}\" is not in a valid encoding, try running with --bin\n")]
    ReadDecodingError(String),
    #[error("IO Error\n")]
    NoMatchesFound,
    #[error("Error writing to file: {0}\n")]
    WriteError(String, #[source] std::io::Error),
    #[error("Could not decode string as hex: {0}\n")]
    HexDecodeError(String, #[source] std::num::ParseIntError),
}

#[derive(Debug)]
pub struct ContentReplacementInfo {
    pub start: usize,
    pub end: usize,
    pub length: usize,
    pub new: String,
    pub original: String,
}

#[derive(Debug)]
pub struct FileReplacementInfo {
    pub did_change: bool,
    pub path: PathBuf,
    pub replacements: Vec<ContentReplacementInfo>,
}

// Instead of a Vec<u8> we will use a Vec<ByteMatcher> to allow for wildcards
pub struct ByteMatcher {
    pub value: u8,
    pub is_wildcard: bool,
}

pub fn do_contents(
    source_path: &Path,
    str_search: &str,
    str_replace: &str,
    b_dry: bool,
    b_bin: bool,
) -> Result<Vec<FileReplacementInfo>> {
    let file = File::open(source_path)?;

    let mut replacement_infos: Vec<FileReplacementInfo> = vec![];

    // Binary search and replace contents
    if b_bin {
        replacement_infos = do_contents_binary(source_path, file, str_search, str_replace, b_dry)?;
    }
    // Plain text search and replace contents
    else {
        replacement_infos = do_contents_plain(source_path, file, str_search, str_replace, b_dry)?;
    }

    Ok(replacement_infos)
}

fn do_contents_plain(
    source_path: &Path,
    file: File,
    str_search: &str,
    str_replace: &str,
    b_dry: bool,
) -> Result<Vec<FileReplacementInfo>> {
    let re = if !str_search.is_empty() {
        Regex::new(&str_search).unwrap()
    } else {
        Regex::new(".*").unwrap()
    };

    // make reader that does BOM sniffing using encoding_rs
    let mut reader = BufReader::new(DecodeReaderBytesBuilder::new().build(file));

    // read file to string
    let mut str_contents = String::new();
    reader
        .read_to_string(&mut str_contents)
        .or(Err(DoContentError::ReadDecodingError(String::from(
            source_path.to_string_lossy(),
        ))))?;

    // search and make sure we have matches

    let search_result = re.find_iter(&str_contents);

    let v_search_result = search_result.collect::<Vec<regex::Match>>();

    if v_search_result.len() == 0 {
        return Ok(vec![]);
    }

    // do replacement and write result

    let result = re.replace_all(&str_contents, str_replace);

    if !b_dry {
        let write_result = fs::write(source_path, result.as_bytes());
        ensure!(
            write_result.is_ok(),
            DoContentError::WriteError(String::from(result), write_result.unwrap_err())
        );
    }

    let mut file_replacement_infos: Vec<FileReplacementInfo> = vec![];

    // log changes, this will make us do re.replace_all twice, but it's fine for now.
    for search_match in v_search_result {
        file_replacement_infos.push(FileReplacementInfo {
            path: source_path.to_path_buf(),
            did_change: !b_dry,
            replacements: vec![ContentReplacementInfo {
                start: search_match.start(),
                end: search_match.end(),
                length: search_match.end() - search_match.start(),
                new: re
                    .replace_all(&search_match.as_str(), str_replace)
                    .to_string(),
                original: search_match.as_str().to_string(),
            }],
        });
    }
    return Ok(file_replacement_infos);
}

fn do_contents_binary(
    source_path: &Path,
    file: File,
    str_search: &str,
    str_replace: &str,
    b_dry: bool,
) -> Result<Vec<FileReplacementInfo>> {
    // decode string hex signature
    let decode_hex_bytes = |s: &str| -> Result<Vec<ByteMatcher>, DoContentError> {
        let split_str = if s.contains("\\x") { "\\x" } else { " " };
        return s
            .split(split_str)
            .filter(|s| !s.is_empty())
            .map(|s| -> Result<ByteMatcher, DoContentError> {
                return u8::from_str_radix(s, 16)
                    .map(|x| ByteMatcher {
                        value: x,
                        is_wildcard: false,
                    })
                    .or_else(|err| {
                        if s == "*" || s == "??" || s == "?" {
                            return Ok(ByteMatcher {
                                value: 0,
                                is_wildcard: true,
                            });
                        }
                        Err(DoContentError::HexDecodeError(
                            String::from(str_search),
                            err,
                        ))
                    });
            })
            .try_collect();
    };

    let search_hex_bytes: Vec<ByteMatcher> = decode_hex_bytes(str_search)?;
    let replace_hex_bytes: Vec<ByteMatcher> = decode_hex_bytes(str_replace)?;

    // read file contents as binary and do replacement
    let reader = BufReader::new(file);
    let mut out_buffer: Vec<u8> = vec![];
    let i: usize = 0;

    let mut matches: Vec<(usize, usize)> = vec![];

    let mut match_start: usize = 0;
    let mut match_end: usize = 0;
    for byte in reader.bytes() {
        let _byte = byte?;
        // if we have a full length match, save it for later
        if (match_end - match_start) == search_hex_bytes.len() {
            matches.push((match_start, match_end));
        }

        // get current match length
        match_end = i;
        if !search_hex_bytes[i].is_wildcard && search_hex_bytes[i].value != _byte {
            match_start += 1; // if it doesn't match we increment match_start
        }

        // always push original byte which we will edit later
        out_buffer.push(_byte);
    }

    let mut file_replacement_infos: Vec<FileReplacementInfo> = vec![];

    // edit bytes with our full length matches
    for m in matches {
        for i in m.0..m.1 {
            // dont do anything if replacement is wildcard or if we shouldn't write changes
            if b_dry || replace_hex_bytes[i - m.0].is_wildcard {
                continue;
            }
            // otherwise write change
            out_buffer[i] = replace_hex_bytes[i - m.0].value;
        }

        file_replacement_infos.push(FileReplacementInfo {
            path: source_path.to_path_buf(),
            did_change: !b_dry,
            replacements: vec![ContentReplacementInfo {
                start: m.0,
                end: m.1,
                length: m.1 - m.0,
                new: str_replace.to_string(),
                original: str_search.to_string(),
            }],
        });
    }

    if !b_dry {
        fs::write(source_path, out_buffer.as_slice())?; // write changes to file
    }

    return Ok(file_replacement_infos);
}
