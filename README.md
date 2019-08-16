# syn

A syntax parser based on the [LLLR] method.

## Requirements
- Rust 1.37.0 or later

## Usage
```bash
syn <INPUT> -g GRAMMAR [-p lllr] [-o OUTPUT]
```

The optional argument `-o` specifies the desired output file for a graph in the [DOT] language.
This is only available with the LR parser.

## Grammar
Grammar files are defined using the [TOML] format.

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

### Ignored tokens
Regular expressions in the `[ignore]` table define tokens that are ignored during syntax analysis.
The patterns need to follow the rules for the `[tokens]` table.

Example:
```toml
[ignore]
whitespace = "[ \t\r\n]*"
comment = "#.*(\n|$)"
```

[LLLR]: https://www.semanticscholar.org/paper/LLLR-Parsing%3A-a-Combination-of-LL-and-LR-Parsing-Slivnik/fac55d573ec8441673022e36f441ca278fc4a717
[DOT]: https://www.graphviz.org/doc/info/lang.html
[TOML]: https://github.com/toml-lang/toml
