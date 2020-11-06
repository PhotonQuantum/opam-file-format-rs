# opam-file-format-rs

Parser for the opam file syntax written in rust.

## Get Started

### Binary

First, let's run a benchmark and sanity check.

``` shell script
$ git clone https://github.com/ocaml/opam-repository
$ find ./opam-repository -name "opam" > opam-files
$ opam-file-format-rs --benchmark ./opam-files
reading files into memory...
parsing files...
parsed 15955 files. elapsed 0.36 secs. speed: 44691.88 files/sec
```

By default, `opam-file-format-rs` outputs the AST of opam files.

``` shell script
$ opam-file-format-rs ./opam
OpamAST {
    items: {
        "opam-version": Variable(
            String(
                "2.0",
            ),
        ),
        ...
    },
}
```

To make your life easier, you may ask `opam-file-format-rs` to output json format.

``` shell script
$ opam-file-format-rs ./opam --json | jq
{
  "opam-version": "2.0",
  "maintainer": "Thomas Gazagnaire <thomas@gazagnaire.org>",
  "authors": [
    "Thomas Gazagnaire <thomas@gazagnaire.org"
  ],
  ...
  "url": {
    "src": "https://github.com/realworldocaml/craml/releases/download/1.0.0/craml-1.0.0.tbz",
    "checksum": "md5=328d4d6bb137054894b215b3e10d95ca"
  }
}
```

Syntax errors in files will be reported. Sometimes the error message or reported position may be useless, but anyway the parser will inform you what's going wrong.

```
$ opam-file-format-rs ./malformed-opam
error: expected `BOOL`, `ENVOP`, `IDENT`, `INT`, `LBRACE`, `LBRACKET`, `LOGOP`, `LPAR`, `PFXOP`, `RBRACE`, `RBRACKET`, `RELOP`, `RPAR`, `STRING`, or end of file
 --> opam:5:13
  | license:      "ISC"
5 | boo: dev-repo: "git+https://github.com/realworldocaml/craml.git"
  |              ^
  | bug-reports:  "https://github.com/realworldocaml/craml/issues"
```

## Library

The document is WIP. You may take a peek at the bin crate to grasp a general idea how it works.
