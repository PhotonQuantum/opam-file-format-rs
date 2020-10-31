# opam-file-format-rs

> WIP
>
> Only lexer is implemented now, and it's even not a lib yet.

Parser for the opam file syntax written in rust.

## Get Started (WIP Lexer)

``` shell script
$ opam-file-format-rs ./opam
[
    IDENT(
        "opam-version",
    ),
    COLON,
    STRING(
        "2.0",
    ),
    IDENT(
        "maintainer",
    ),
    COLON,
    STRING(
        "Thomas Gazagnaire <thomas@gazagnaire.org>",
    ),
    ...
    COLON,
    STRING(
        "https://github.com/realworldocaml/craml/releases/download/1.0.0/craml-1.0.0.tbz",
    ),
    IDENT(
        "checksum",
    ),
    COLON,
    STRING(
        "md5=328d4d6bb137054894b215b3e10d95ca",
    ),
    RBRACE,
]
```
