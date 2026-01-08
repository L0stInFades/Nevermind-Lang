//! Source code location tracking

use std::fmt;
use std::path::PathBuf;

/// A location in source code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    /// File path (None for stdin or strings)
    pub file: Option<PathBuf>,

    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed, in UTF-8 characters)
    pub column: usize,

    /// Absolute byte offset from the start of the file
    pub offset: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file: Option<PathBuf>, line: usize, column: usize, offset: usize) -> Self {
        Self {
            file,
            line,
            column,
            offset,
        }
    }

    /// Create a location for stdin or a string
    pub fn anonymous() -> Self {
        Self {
            file: None,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Create a location at the start of a file
    pub fn start_of_file(file: PathBuf) -> Self {
        Self {
            file: Some(file),
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Advance the location by one character
    pub fn advance_char(&mut self, c: char) {
        self.offset += c.len_utf8();
        self.column += 1;

        if c == '\n' {
            self.line += 1;
            self.column = 1;
        }
    }

    /// Advance the location by a string
    pub fn advance_str(&mut self, s: &str) {
        for c in s.chars() {
            self.advance_char(c);
        }
    }

    /// Create an EOF location at this position
    pub fn to_eof(&self) -> Self {
        Self {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            offset: self.offset,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:{}:{}", file.display(), self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self::anonymous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_char() {
        let mut loc = SourceLocation::anonymous();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);

        loc.advance_char('a');
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 2);

        loc.advance_char('\n');
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn test_advance_str() {
        let mut loc = SourceLocation::anonymous();
        loc.advance_str("hello\nworld");

        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 6);
    }
}
