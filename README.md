# syn

## Requirements
- Rust 1.31.0 or later

## Usage
```bash
cargo run grammar/<grammar>.toml [-o <grammar>.dot]
```

The `-o` argument is optional and specifies the desired output file for a graph in the
[DOT](https://www.graphviz.org/doc/info/lang.html) language.

## Grammar
Grammar files are defined using the [TOML](https://github.com/toml-lang/toml) format.

### Header
The header contains the following entries:

- `name`: Name of the grammar.
- `description`: An optional description of the grammar.
  Defaults to the canonical path to the grammar file.
- `start_symbol`: Start symbol of the grammar. Defaults to first rule in `[rules]`.

Example:
```toml
name = "grammar"
description = "Example grammar for README"
start_symbol = "S"
```

### Rules
The production rules are described in the `[rules]` table. A production can either be a single
string, or an array of strings, each representing the possible rules for the specific grammar
symbol.  When parsing the grammar file, a single string is converted to an array with one element.

To represent an `ϵ` production, use an empty string. The symbols and rules can be in any order.

Example:
```toml
[rules]
# S → A B 'c' | 'a' A B 'b'
S = [
    "A B c",
    "a A B b",
]

# A → 'a' | ϵ
A = [
    "a",
    "",
]

# B → 'b'
B = "b"
```

### Tokens
Regular expressions to match tokens during lexical analysis are described in the `[tokens]` table.
The patterns need to be properly escaped and written in a way that allows partial matching
for the incremental lexical analysis.

Matching precedence is defined by the order of the regular expressions.

Example:
```toml
[tokens]
a = "(true|false)"
b = "'[A-Z\\x61-\\x7A_]*('|$)"
c = "[0-9]+"
```
