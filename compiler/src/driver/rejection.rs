//! The two facts every source rejection carries at the driver boundary: where
//! it is, in the terms the caller typed, and what the compiler was reading when
//! it stopped.
//!
//! A stage reports a rejection as a byte coordinate, because a byte coordinate
//! is what the judgment has. A byte coordinate is not what a writer has: the
//! blind-writer trial of 2026-08-28 recorded a writer running `head -c` on
//! their own program to find out which construct `ByteOffset(11951)` meant, and
//! the judge bisecting two more the same way. The bytes and the source are both
//! here, so the line is free to print.
//!
//! Nothing here is a rendering redesign. The detail text is still one stage
//! value's `Debug`; this only wraps that value with the location and the line,
//! so a reader gets a sentence instead of an offset.

use core::fmt;

use crate::SyntaxCoordinate;
use crate::source::SourceBundle;

/// One stage rejection, its location in host terms, and its source line.
pub(super) struct Located<Issue> {
    issue: Issue,
    at: String,
    source_line: String,
}

impl<Issue: fmt::Debug> fmt::Debug for Located<Issue> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} at {} in line {:?}",
            self.issue, self.at, self.source_line
        )
    }
}

impl<Issue> Located<Issue> {
    /// Wraps one rejection with the source context its coordinate names.
    ///
    /// A coordinate the bundle cannot resolve leaves the location and the line
    /// empty rather than failing the compilation: this is presentation, and a
    /// stage that already has a verdict must still deliver it.
    pub(super) fn new(issue: Issue, bundle: &SourceBundle, coordinate: SyntaxCoordinate) -> Self {
        let (at, source_line) = context(bundle, coordinate)
            .unwrap_or_else(|| ("an unresolved coordinate".to_owned(), String::new()));
        Self {
            issue,
            at,
            source_line,
        }
    }
}

/// The `path:line:column` of one coordinate and the whole source line holding
/// it.
///
/// The path is the display path, so a rejection names the file the caller
/// named. Line and column are one-based, counted in bytes: the language's
/// source is ASCII [FORM-3], so a byte column is a column.
fn context(bundle: &SourceBundle, coordinate: SyntaxCoordinate) -> Option<(String, String)> {
    let file = bundle.file(coordinate.source())?;
    let bytes = file.bytes();
    let start = usize::try_from(coordinate.start().value()).ok()?;
    let start = start.min(bytes.len());
    let line_start = bytes[..start]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index.saturating_add(1));
    let line_end = bytes[line_start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| line_start.saturating_add(offset));
    let line = bytes[..line_start]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        .saturating_add(1);
    let column = start.saturating_sub(line_start).saturating_add(1);
    let source_line = String::from_utf8_lossy(bytes.get(line_start..line_end)?).into_owned();
    Some((
        format!("{}:{line}:{column}", file.display_path()),
        source_line,
    ))
}
