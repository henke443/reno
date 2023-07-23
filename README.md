# Renovate
Name is short for *Renovate*. 

A simple but powerful command-line batch file search-and-replace tool that is efficient and cross-platform.

# Features
- Search and replace filenames using regex
- Search and replace file contents using regex
- Regex capture groups
- Binary search and replace using wildcard signatures
- Globs
- Multithreaded & written in rust

## To see it in action run
`cargo run -- "test(\.md|\.txt)" "changed_test${1}"` and then check the filenames and contents of test.txt and test.md.

## More examples:
`reno "^(FolderPrefix?)([^\.]*)$" "${2}" --names` - Recursively removes the string FolderPrefix in the beginning of all folder names

`reno "DE ?? BE EF" "FF FF ?? ??" --bin -g test.bin` - Edits the binary "DE AD BE EF" segments of the test.bin file to "FF FF BE EF"

## Dangerous scenarios:
You should always run `--dry` before you let reno actually replace anything.
For example, if you run `reno -g *test.* -R "changed_test" --names` then `test.txt` becomes changed_test BUT `test.md` ALSO BECOMES changed_test, leading to one of them being overwritten.
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
