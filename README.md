# ftb

![CI](https://github.com/acamino/ftb/actions/workflows/ci.yml/badge.svg)

A simple CLI tool to format and align Markdown tables.

This is a Rust port of the JavaScript formatter from [markdowntable.com](http://markdowntable.com/).

## Installation

### From crates.io

```bash
cargo install ftb
```

This installs the `ftb` binary.

### From source

```bash
git clone https://github.com/acamino/ftb
cd ftb
cargo install --path .
```

## Usage

Pipe a Markdown table through `ftb` to align its columns:

```bash
pbpaste | ftb
```

Or use with files:

```bash
cat table.md | ftb
ftb table.md
ftb table.md > formatted.md
```

Try it with the demo file:

```bash
ftb examples/demo.md
```

## Examples

### Basic Table

Input:
```
| h1 | h2 | h3 |
|-|-|-|
| data1 | data2 | data3 |
```

Output:
```
| h1    | h2    | h3    |
|-------|-------|-------|
| data1 | data2 | data3 |
```

### Irregular Table

Input:
```
h1 | h2 | h3
-|-|-
data-1 | data-2 | data-3
```

Output:
```
| h1     | h2     | h3     |
|--------|--------|--------|
| data-1 | data-2 | data-3 |
```

### Complex Table with Missing Cells

Input:
```
| Header 1 | Header 2 | Header 3 |
|----|---|-|
| data1a | Data is longer than header | 1 |
| d1b | add a cell|
|lorem|ipsum|3|
```

Output:
```
| Header 1 | Header 2                   | Header 3 |
|----------|----------------------------|----------|
| data1a   | Data is longer than header | 1        |
| d1b      | add a cell                 |          |
| lorem    | ipsum                      | 3        |
```

## Features

- Aligns columns based on content width
- Handles missing cells by adding empty columns
- Removes leading/trailing empty columns
- Works with irregular table formats
- Fast and lightweight

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

The binary will be available at `target/release/ftb`.

### Local Development Setup

For ergonomic local testing, choose one of these options:

**Option 1: Install locally**
```bash
cargo install --path .
# Now use: ftb
```

**Option 2: Symlink to /usr/local/bin**
```bash
ln -s $(pwd)/target/release/ftb /usr/local/bin/ftb
# Now use: ftb anywhere
```

## License

MIT

## Credits

Port of the Markdown table formatter from [markdowntable.com](http://markdowntable.com/).
