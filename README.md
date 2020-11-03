# opam-file-format-rs

> WIP
>
> This is only a demo, and there are known issues in lexer.

Parser for the opam file syntax written in rust.

## Get Started (Demo)

``` shell script
$ opam-file-format-rs ./opam
OpamFile {
    items: [
        Variable(
            "opam-version",
            String(
                "2.0",
            ),
        ),
        ...
        Section {
            kind: "url",
            name: None,
            items: [
                Variable(
                    "src",
                    String(
                        "https://github.com/realworldocaml/craml/releases/download/1.0.0/craml-1.0.0.tbz",
                    ),
                ),
                Variable(
                    "checksum",
                    String(
                        "md5=328d4d6bb137054894b215b3e10d95ca",
                    ),
                ),
            ],
        },
    ],
}

$ opam-file-format-rs ./malformed-opam
error: expected `BOOL`, `ENVOP`, `IDENT`, `INT`, `LBRACE`, `LBRACKET`, `LOGOP`, `LPAR`, `PFXOP`, `RBRACE`, `RBRACKET`, `RELOP`, `RPAR`, `STRING`, or end of file
 --> opam:5:13
  | license:      "ISC"
5 | boo: dev-repo: "git+https://github.com/realworldocaml/craml.git"
  |              ^
  | bug-reports:  "https://github.com/realworldocaml/craml/issues"
```
