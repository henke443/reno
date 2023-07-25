![rust workflow](https://github.com/henke443/reno/actions/workflows/rust.yml/badge.svg)

# Renovate
Name is short for *Renovate*. 

A simple but powerful command-line batch file search-and-replace tool that is efficient and cross-platform.

# Features
- Search and replace file and folder names using regex
- Search and replace file contents using regex
- Regex capture groups
- Binary search and replace using wildcard signatures
- Globs
- Multithreaded & written in rust

## To see it in action run
`cargo run -- "test(\.md|\.txt)" "changed_test${1}"` and then check the filenames and contents of test.txt and test.md.

## More examples:
`reno "^(FolderPrefix?)([^\.]*)$" "${2}" --names` - Recursively removes the string FolderPrefix in the beginning of all folder names.

`reno "DE ?? BE EF" "00 00 ?? ??" --bin -g test.bin` - Changes the bytes in the example file from:

```
DE AD BE EF
DE AD BE EF
01 02 03 04
05 06 07 08
09 10 11 12
13 14 15 DE
AD BE EF
```

To

```
00 00 BE EF
00 00 BE EF
01 02 03 04
05 06 07 08
09 10 11 12
13 14 15 00
00 BE EF
```

## Dangerous scenarios:
You should always run `--dry` before you let reno actually replace anything.
For example, if you run `reno ".*" "changed_test" -g *test.* --names` then `test.txt` becomes changed_test BUT `test.md` ALSO BECOMES changed_test, leading to one of them being overwritten.
This is somewhat of a bug and there will be some checks in place so that this doesn't happen.



`> reno --help`

```
A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.

Usage: reno.exe [OPTIONS] <SEARCH> [REPLACE]

Arguments:
  <SEARCH>
          Search regex or binary sequence if --bin is passed.

          In the binary mode, the search string should be a binary sequence with optional wildcards (e.g.: "\x22\x??\x??\x44\x22\x01\x69\x55" or "22 ?? ?? 44 22 01 69 55"))

  [REPLACE]
          Regex (e.g.: "Hello ${1}") in the normal mode.

          **IMPORTANT**: Even though capture groups without curly braces (for example just $1 instead of ${1}) mostly work, I strongly advise using them as unexpected results can occur otherwise.

          Be sure to always run --dry before you actually replace anything.

          A binary sequence (e.g.: "\x22\x01\xD5\x44\x22\x01\x69\x55") in binary mode.

          Dry mode if left empty.

Options:
      --dry
          Don't modify files, just show what would happen

  -g, --globs <GLOBS>
          Filename glob patterns, defaults to: "*"

          [default: **]

  -b, --bin
          Binary search and replace mode

  -c, --contents
          Only search and replace file contents

  -n, --names
          Only search and replace file names

  -d, --depth <DEPTH>
          Max depth of directory traversal. 0 means only current directory

          [default: 4294967294]

  -v, --verbose
          Only search and replace file names

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Planned features / TODO:

### Add more warnings and safety checks
- If two files are renamed to the same name [ ]
- By default the user should be prompted to accept dry run results before actual changes are ran, at the very least batches of the results. [ ]

### Binary regex
`reno "4C 79 72 61 [utf8:[A-z0-9]{10}]*10" "00 00 00 00 [??]..." --bin -g test.bin` - To be able to safely edit text in binary files it would be nice to have some way of knowing that the text is followed by (or prepended/surrounded) by some number of valid ascii characters or utf8 characters. A simple approach like this would probably be best. The three features I'm planning currently are: 
1. [encoding_name:regex]*length bracket syntax
2. [anything]... triple dots meaning any number of repetitions
3. [anything]*length bracket repetition syntax
  
### Modifying globwalk.rs?
It doesn't really support multithreading atm so I use `.par_bridge()`


## Testing out some github badges
[![Henrik's GitHub stats](https://github-readme-stats.vercel.app/api?username=henke443)](https://github.com/anuraghazra/github-readme-stats)
![Top Langs](https://github-readme-stats.vercel.app/api/top-langs/?username=anuraghazra&hide_progress=true)






