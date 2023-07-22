To see it in action run
`cargo run -- -S "test(\.md|\.txt)" -R "changed_test$1" -G *.txt -G *.md`

```
A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.

Usage: reno.exe [OPTIONS]

Options:
  -G, --glob-patterns <GLOB_PATTERNS>
          Filename glob patterns, defaults to: "./*.*"
  -S <SEARCH_REGEX>
          Search regex
  -R <REPLACE_REGEX>
          Replace regex
  -d, --dry
          Don't modify files, just show what would happen
  -c, --contents
          Only search and replace file contents
  -n, --names
          Only search and replace file names
  -m, --max-depth <MAX_DEPTH>
          Max depth of directory traversal, unlimited by default. 0 means only the current directory
  -h, --help
          Print help
  -V, --version
          Print version
```
