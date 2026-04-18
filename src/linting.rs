use std::error::Error;
use std::path::{Path, PathBuf};

use crate::formatting;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LintWarning {
    line: usize,
    message: String,
}

/// Lint one or more Nevermind files.
pub fn lint_paths(inputs: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    if inputs.is_empty() {
        return Err("no input files provided".into());
    }

    let mut total_warnings = 0usize;

    for input in &inputs {
        let source = std::fs::read_to_string(input)?;
        let warnings = lint_source(input, &source)?;

        if warnings.is_empty() {
            println!("clean: {}", input.display());
            continue;
        }

        for warning in &warnings {
            println!("{}:{}: {}", input.display(), warning.line, warning.message);
        }
        total_warnings += warnings.len();
    }

    if total_warnings > 0 {
        return Err(format!("lint found {} issue(s)", total_warnings).into());
    }

    Ok(())
}

fn lint_source(path: &Path, source: &str) -> Result<Vec<LintWarning>, Box<dyn Error>> {
    let mut lexer = nevermind_lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let statements = parser.parse()?;

    let base_dir = path
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .or_else(|| path.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut resolver = nevermind_name_resolver::NameResolver::with_base_dir(base_dir);
    if let Err(errors) = resolver.resolve(&statements) {
        let message = errors
            .iter()
            .map(|error| format!("{}: {}", error.span, error.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("Name resolution failed: {}", message).into());
    }

    let mut checker = nevermind_type_checker::TypeChecker::new();
    checker.check(&statements)?;

    let mut warnings = Vec::new();

    if source != formatting::format_source(source)? {
        warnings.push(LintWarning {
            line: 1,
            message: "file is not formatted; run `nevermind fmt --write <file>`".to_string(),
        });
    }

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let leading = line
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<String>();

        if leading.contains('\t') {
            warnings.push(LintWarning {
                line: line_number,
                message: "tabs are not allowed for indentation; use spaces".to_string(),
            });
        }

        if line.ends_with(' ') || line.ends_with('\t') {
            warnings.push(LintWarning {
                line: line_number,
                message: "trailing whitespace".to_string(),
            });
        }

        if line.chars().count() > 100 {
            warnings.push(LintWarning {
                line: line_number,
                message: "line exceeds 100 characters".to_string(),
            });
        }

        if line.contains("TODO") || line.contains("FIXME") {
            warnings.push(LintWarning {
                line: line_number,
                message: "leftover TODO/FIXME marker".to_string(),
            });
        }
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::lint_source;

    #[test]
    fn lint_reports_style_issues() {
        let source = "fn main() do\n    print \"hi\"  # TODO cleanup   \nend\n";
        let warnings = lint_source(Path::new("sample.nm"), source).unwrap();

        assert!(warnings
            .iter()
            .any(|warning| warning.message.contains("not formatted")));
        assert!(warnings
            .iter()
            .any(|warning| warning.message.contains("TODO/FIXME")));
        assert!(warnings
            .iter()
            .any(|warning| warning.message.contains("trailing whitespace")));
    }

    #[test]
    fn lint_accepts_clean_source() {
        let source = "fn main() do\n  print \"hi\"\nend\n";
        let warnings = lint_source(Path::new("sample.nm"), source).unwrap();
        assert!(warnings.is_empty());
    }
}
