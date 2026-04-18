use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Format one or more Nevermind files.
pub fn format_paths(inputs: Vec<PathBuf>, write: bool, check: bool) -> Result<(), Box<dyn Error>> {
    if inputs.is_empty() {
        return Err("no input files provided".into());
    }

    let multiple_inputs = inputs.len() > 1;
    let mut needs_formatting = Vec::new();

    for (index, input) in inputs.iter().enumerate() {
        let source = fs::read_to_string(input)?;
        let formatted = format_source(&source)?;
        let is_changed = source != formatted;

        if check {
            if is_changed {
                println!("needs formatting: {}", input.display());
                needs_formatting.push(input.display().to_string());
            } else {
                println!("already formatted: {}", input.display());
            }
            continue;
        }

        if write {
            if is_changed {
                fs::write(input, formatted)?;
                println!("formatted: {}", input.display());
            } else {
                println!("already formatted: {}", input.display());
            }
            continue;
        }

        if multiple_inputs {
            if index > 0 {
                println!();
            }
            println!("==> {} <==", input.display());
        }

        print!("{}", formatted);
    }

    if !needs_formatting.is_empty() {
        return Err(format!("{} file(s) need formatting", needs_formatting.len()).into());
    }

    Ok(())
}

/// Format a Nevermind source string while preserving comments.
pub fn format_source(source: &str) -> Result<String, Box<dyn Error>> {
    validate_syntax(source)?;
    Ok(reindent_source(source))
}

fn validate_syntax(source: &str) -> Result<(), Box<dyn Error>> {
    let mut lexer = nevermind_lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let _ = parser.parse()?;
    Ok(())
}

fn reindent_source(source: &str) -> String {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = Vec::new();
    let mut indent_level = 0usize;
    let mut saw_content = false;
    let mut previous_blank = false;

    for raw_line in normalized.lines() {
        let trimmed_end = raw_line.trim_end();
        let trimmed_start = trimmed_end.trim_start();

        if trimmed_start.is_empty() {
            if saw_content && !previous_blank {
                lines.push(String::new());
                previous_blank = true;
            }
            continue;
        }

        let structural = sanitize_for_structure(trimmed_start);
        let (dedent_before, remaining) = strip_leading_dedent(&structural);
        indent_level = indent_level.saturating_sub(dedent_before);

        lines.push(format!("{}{}", indent(indent_level), trimmed_start));
        saw_content = true;
        previous_blank = false;

        let delta = indentation_delta(remaining);
        indent_level = ((indent_level as isize) + delta).max(0) as usize;
    }

    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn strip_leading_dedent(code: &str) -> (usize, &str) {
    let code = code.trim_start();

    if let Some(rest) = code.strip_prefix('}') {
        return (1, rest.trim_start());
    }

    for keyword in ["end", "else", "elif"] {
        if starts_with_keyword(code, keyword) {
            return (1, code[keyword.len()..].trim_start());
        }
    }

    (0, code)
}

fn indentation_delta(code: &str) -> isize {
    count_keyword(code, "do") as isize + count_keyword(code, "then") as isize
        - count_keyword(code, "end") as isize
        + brace_delta(code)
}

fn starts_with_keyword(code: &str, keyword: &str) -> bool {
    code.strip_prefix(keyword)
        .map(|rest| {
            rest.is_empty()
                || !rest
                    .chars()
                    .next()
                    .map(is_identifier_continue)
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn count_keyword(code: &str, keyword: &str) -> usize {
    let mut count = 0;
    let mut index = 0;

    while let Some(found) = code[index..].find(keyword) {
        let start = index + found;
        let end = start + keyword.len();

        let before_ok = start == 0
            || !code[..start]
                .chars()
                .next_back()
                .map(is_identifier_continue)
                .unwrap_or(false);
        let after_ok = end >= code.len()
            || !code[end..]
                .chars()
                .next()
                .map(is_identifier_continue)
                .unwrap_or(false);

        if before_ok && after_ok {
            count += 1;
        }

        index = end;
    }

    count
}

fn brace_delta(code: &str) -> isize {
    let mut delta = 0isize;
    for ch in code.chars() {
        match ch {
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn sanitize_for_structure(line: &str) -> String {
    let mut sanitized = String::new();
    let mut chars = line.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_double {
            sanitized.push(' ');
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_double = false;
            }
            continue;
        }

        if in_single {
            sanitized.push(' ');
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_single = false;
            }
            continue;
        }

        if ch == '#' {
            break;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            break;
        }

        if ch == '"' {
            in_double = true;
            sanitized.push(' ');
            continue;
        }

        if ch == '\'' {
            in_single = true;
            sanitized.push(' ');
            continue;
        }

        sanitized.push(ch);
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::format_source;

    #[test]
    fn formatter_normalizes_indentation_and_whitespace() {
        let source = "fn main() do\n    print \"hi\"   \n\n\n  # note\n    if true do\n      print \"nested\"\n    end\nend\n";
        let formatted = format_source(source).unwrap();

        assert_eq!(
            formatted,
            "fn main() do\n  print \"hi\"\n\n  # note\n  if true do\n    print \"nested\"\n  end\nend\n"
        );
    }

    #[test]
    fn formatter_preserves_comments_and_match_blocks() {
        let source = "fn describe(x) do\nmatch x {\nSome(v) => print \"ok\",\n# fallback\n_ => print \"nope\"\n}\nend\n";
        let formatted = format_source(source).unwrap();

        assert_eq!(
            formatted,
            "fn describe(x) do\n  match x {\n    Some(v) => print \"ok\",\n    # fallback\n    _ => print \"nope\"\n  }\nend\n"
        );
    }
}
