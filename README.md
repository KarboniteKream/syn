# syn

## Requirements
- Rust 1.32.0 or later

## Usage
```bash
cargo run grammar/<grammar>.toml
```

## Grammar
Grammar files are defined using the TOML format.

### Header
The header contains the following entries:

- `name`: Name of the grammar.
- `description`: An optional description of the grammar. Defaults to the filename.
- `start_symbol`: Start symbol of the grammar. Defaults to `"S"`.

Example:
```toml
name = "grammar"
description = "Example grammar for README"
start_symbol = "S"
```

### Rules
The production rules are described in a `[rules]` table. A production can either be a single string,
or an array of strings, each representing the possible rules for the specific grammar symbol.
When parsing the grammar file, a single string is converted to an array with one element.

To represent an `ϵ` production, use an empty string. The symbols and rules can be in any order.

Example:
```toml
# S → A B c | a A B b
S = [
    "A B c",
    "a A B b",
]

# A → a | ϵ
A = [
    "a",
    "",
]

# B → b
B = "b"
```
