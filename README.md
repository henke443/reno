Name is short for *Renovate*. A simple but powerful command-line batch file editor. Enables you to use regex search and replace on both filenames and contents, efficiently (multi-threaded) and cross-platform.

To see it in action run
`cargo run -- -S "test(\.md|\.txt)" -R "changed_test${1}"` and then check the filenames and contents of test.txt and test.md.

More examples:
`reno -G *test.* -R "changed_test" --names` - test.txt becomes changed_test and test.md becomes changed_test, one of them will be overwritten depending on which thread wins.
Is somewhat of a bug and I'll do some check for it but in the meantime be aware of that.

`reno -S "^(FolderPrefix?)([^\.]*)$" -R ${2} --names` - Recursively removes the string FolderPrefix in the beginning of all folder names


`> reno --help`
```
A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.

Usage: reno.exe [OPTIONS] <SEARCH>

Arguments:
  <SEARCH>
          Search regex or binary sequence if --bin is passed.

          In the binary mode, the search string should be a binary sequence with optional wildcards (e.g.: "\x22\x??\x??\x44\x22\x01\x69\x55" or "22 ?? ?? 44 22 01 69 55"))
                
Options:
  -R <REPLACE>
          Either regex (e.g.: "Hello ${1}") in the normal mode, or a binary sequence (e.g.: "\x22\x01\xD5\x44\x22\x01\x69\x55") in binary mode. Dry mode if left empty

      --dry
          Don't modify files, just show what would happen

  -G, --globs <GLOBS>
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
