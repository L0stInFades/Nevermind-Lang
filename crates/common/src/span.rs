//! Span tracking for source code ranges

use std::fmt;
use std::ops::Range;

use crate::SourceLocation;

/// A span in source code (from start to end, inclusive)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl Span {
    /// Create a new span
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    /// Create a span for a single location (zero-length)
    pub fn point(loc: SourceLocation) -> Self {
        Self {
            start: loc.clone(),
            end: loc,
        }
    }

    /// Create a dummy span for synthetic nodes
    pub fn dummy() -> Self {
        Self::point(SourceLocation::anonymous())
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if span is zero-length
    pub fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }

    /// Merge two spans
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.clone(),
            end: other.end.clone(),
        }
    }

    /// Extend this span to include another
    pub fn extend(&mut self, other: &Span) {
        self.end = other.end.clone();
    }

    /// Get the file path (if any)
    pub fn file(&self) -> Option<&std::path::Path> {
        self.start.file.as_deref()
    }

    /// Get the line range
    pub fn line_range(&self) -> Range<usize> {
        self.start.line..self.end.line
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

impl From<SourceLocation> for Span {
    fn from(loc: SourceLocation) -> Self {
        Self::point(loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_len() {
        let start = SourceLocation::anonymous();
        let mut end = start.clone();
        end.advance_str("hello");

        let span = Span::new(start, end);
        assert_eq!(span.len(), 5);
    }

    #[test]
    fn test_span_merge() {
        let loc1 = SourceLocation::anonymous();
        let mut loc2 = loc1.clone();
        loc2.advance_str("hello");
        let mut loc3 = loc2.clone();
        loc3.advance_str(" world");

        let span1 = Span::new(loc1.clone(), loc2.clone());
        let span2 = Span::new(loc2, loc3);
        let merged = span1.merge(&span2);

        assert_eq!(merged.len(), 11);
    }
}
