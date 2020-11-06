use colored::*;

use opam_file_format::lexer;

pub fn pretty_error(filename: &str, source: &str, span: &lexer::Span, message: &str) {
    let split_source: Vec<&str> = source.lines().collect();
    let lexer::Span { start, end } = span.clone();
    let (start_ln, end_ln) = get_line_range(source, start, end);
    let pad = (&end_ln.line).to_string().chars().count();
    eprintln!("{}", format!("{}: {}", "error".red(), message).bold());
    eprintln!(
        "{}{} {}:{}:{}",
        repeat_str(" ", pad),
        "-->".blue().bold(),
        filename,
        (&start_ln.line),
        (&start_ln.col)
    );
    let prefix = format!("{} |", start_ln.line).blue().bold();
    let prefix_wo_ln = format!("{} |", repeat_str(" ", pad)).blue().bold();
    if start_ln.line != 0 {
        eprintln!("{} {}", prefix_wo_ln, split_source[start_ln.line - 1]);
    }
    eprintln!("{} {}", prefix, split_source[start_ln.line]);
    let line_length = split_source[start_ln.line].chars().count();
    let mark_len = if line_length - start_ln.col < span.end - span.start {
        line_length - start_ln.col
    } else {
        span.end - span.start
    };
    eprintln!(
        "{} {}{}",
        prefix_wo_ln,
        repeat_str(" ", start_ln.col),
        repeat_str("^", mark_len).red().bold()
    );
    if end_ln.line < split_source.len() {
        eprintln!("{} {}", prefix_wo_ln, split_source[start_ln.line + 1]);
    }
}

struct Pos {
    line: usize,
    col: usize,
}

fn repeat_str(s: &str, n: usize) -> String {
    (0..n).map(|_| s).collect()
}

fn get_line_range(source: &str, start: usize, end: usize) -> (Pos, Pos) {
    let mut current_pos: usize = 0;
    let mut start_ln: Option<Pos> = None;
    let mut end_ln: Option<Pos> = None;
    for (i, line) in source.lines().enumerate() {
        let line_length = line.chars().count();
        if let None = start_ln {
            if start >= current_pos && start < current_pos + line_length {
                start_ln = Some(Pos {
                    line: i,
                    col: start - current_pos,
                });
            }
        }
        if let None = end_ln {
            if end >= current_pos && end <= current_pos + line_length {
                end_ln = Some(Pos {
                    line: i,
                    col: end - current_pos,
                });
                break;
            }
        }
        current_pos += line_length + 1;
    }
    (
        start_ln.expect("Unable to parse line number"),
        end_ln.expect("Unable to parse line number"),
    )
}
