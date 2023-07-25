extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::ensure;
use anyhow::Result;
use encoding_rs_io::DecodeReaderBytesBuilder;
use itertools::Itertools;
use regex::Regex;
use std::borrow::BorrowMut;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{BufReader, Read};
//use std::os::windows::prelude::FileExt;
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
#[derive(Debug)]
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
    let mut file = OpenOptions::new().read(true).write(true).open(source_path)?;

    let mut replacement_infos: Vec<FileReplacementInfo> = vec![];

    // Binary search and replace contents
    if b_bin {
        replacement_infos =
            do_contents_binary(source_path, &mut file, str_search, str_replace, b_dry)?;
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
    file: &mut File,
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
                        if s == "??" {
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
    let mut reader = BufReader::new(file.try_clone()?);
    let mut i: usize = 0;

    let mut file_replacement_infos: Vec<FileReplacementInfo> = vec![];

    let replace_length = replace_hex_bytes.len();
    let mut potential_match: Vec<u8> = vec![0u8; search_hex_bytes.len()];

    //println!("search_hex_bytes: {:?}", search_hex_bytes);
    //println!("replace_hex_bytes: {:?}", replace_hex_bytes);

    loop {
        match reader.read_exact(&mut potential_match) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => break,
                _ => {
                    panic!("Error reading file: {:?}", e);
                }
            },
        };

        //println!("buffer: {:?}", potential_match);
        let mut matched = true;
        for (search_i, search_byte) in search_hex_bytes.iter().enumerate() {
            if (search_byte.value != potential_match[search_i] && !search_byte.is_wildcard) {
                matched = false;
                //println!("i: {} {} != {}", i, search_byte.value, potential_match[search_i]);
            }
        }

        if (matched) { 
            for (matched_i, matched_byte) in potential_match.iter_mut().enumerate() {
                if !replace_hex_bytes[matched_i].is_wildcard {
                    *matched_byte = replace_hex_bytes[matched_i].value;
                }
            }
        
        }

        if !b_dry && matched {
            file.seek(SeekFrom::Start(i as u64));
            file.write_all(&potential_match);
            //file.seek_write(&potential_match, i as u64).unwrap_or_else(|e| panic!("Could not seek_write: {:?}", e));
        }

        if (matched) {
            file_replacement_infos.push(FileReplacementInfo {
                path: source_path.to_path_buf(),
                did_change: !b_dry,
                replacements: vec![ContentReplacementInfo {
                    start: i,
                    end: i + replace_length,
                    length: replace_length,
                    new: str_replace.to_string(),
                    original: str_search.to_string(),
                }],
            });
        }

        reader.seek_relative(-(potential_match.len() as i64) + 1)?;
        i += 1;
    }

    return Ok(file_replacement_infos);
}
