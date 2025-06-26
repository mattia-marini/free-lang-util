# Parsing utils for free grammars
A tool for generating parsing tables and automaton of free grammars, written in rust 🦀

## Usage
```bash
Usage: latex-parsing-table-generator [OPTIONS] <--file <FILE>|--base-64 <BASE64>>
```

where `FILE` should be a valid path to a file containing a grammar with the same format used on [grammophone](https://mdaines.github.io/grammophone/#/), for instance:
```
S -> A C .
A -> a S B | .
B -> b A | B b | D .
C -> c S C | .
D -> d D | .
```
or alternatively, ca base64 representation of the string encoding the grammar could be provided using the `--base-64` flag
