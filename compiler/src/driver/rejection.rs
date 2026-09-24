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

use super::SourceLocation;
use crate::SyntaxCoordinate;
use crate::source::SourceBundle;

/// One stage rejection, its location in host terms, and its source line.
pub(super) struct Located<Issue> {
    issue: Issue,
    at: Option<SourceLocation>,
    source_line: String,
    /// Where the concrete generic instance whose check failed was requested,
    /// and that line, when the rejection arose in one [FN-2, MOD-8].
    requested: Option<(SourceLocation, String)>,
}

impl<Issue: fmt::Debug> fmt::Debug for Located<Issue> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.at {
            Some(at) => write!(
                formatter,
                "{:?} at {at} in line {:?}",
                self.issue, self.source_line
            )?,
            None => write!(
                formatter,
                "{:?} at an unresolved coordinate in line {:?}",
                self.issue, self.source_line
            )?,
        }
        if let Some((at, line)) = &self.requested {
            write!(
                formatter,
                ", in the instance requested at {at} in line {line:?}"
            )?;
        }
        Ok(())
    }
}

impl<Issue> Located<Issue> {
    /// Wraps one rejection with the source context its coordinate names.
    ///
    /// A coordinate the bundle cannot resolve leaves the location and the line
    /// empty rather than failing the compilation: this is presentation, and a
    /// stage that already has a verdict must still deliver it.
    pub(super) fn new(issue: Issue, bundle: &SourceBundle, coordinate: SyntaxCoordinate) -> Self {
        Self::at(issue, bundle, coordinate, Anchor::Start)
    }

    /// Wraps one rejection whose coordinate is a trivia gap between two
    /// terminals rather than a written construct.
    ///
    /// A gap that carries a line break begins at the end of the line before
    /// the one the writer must edit, so anchoring at its start quoted the
    /// enclosing item's header while the offending bytes sat two lines down.
    /// The line to quote is the one the gap ends in, and the column is where
    /// the gap's own bytes begin on that line.
    pub(super) fn in_gap(
        issue: Issue,
        bundle: &SourceBundle,
        coordinate: SyntaxCoordinate,
    ) -> Self {
        Self::at(issue, bundle, coordinate, Anchor::LastLineOfGap)
    }

    fn at(
        issue: Issue,
        bundle: &SourceBundle,
        coordinate: SyntaxCoordinate,
        anchor: Anchor,
    ) -> Self {
        let (at, source_line) = match context(bundle, coordinate, anchor) {
            Some((at, line)) => (Some(at), line),
            None => (None, String::new()),
        };
        Self {
            issue,
            at,
            source_line,
            requested: None,
        }
    }

    /// Wraps one rejection with a location already rendered from another
    /// record, such as a graph entry's.
    pub(super) const fn written(issue: Issue, at: SourceLocation, source_line: String) -> Self {
        Self {
            issue,
            at: Some(at),
            source_line,
            requested: None,
        }
    }

    /// Adds the call that requested the concrete generic instance whose
    /// check produced this rejection: the rejection stays at the template,
    /// which owns it, and names who asked for the instance [FN-2, MOD-8].
    pub(super) fn requested_at(
        mut self,
        bundle: &SourceBundle,
        coordinate: Option<SyntaxCoordinate>,
    ) -> Self {
        self.requested =
            coordinate.and_then(|coordinate| context(bundle, coordinate, Anchor::Start));
        self
    }
}

/// The `path:line:column` of a written construct's first byte and its line.
pub(super) fn written_at(
    bundle: &SourceBundle,
    coordinate: SyntaxCoordinate,
) -> Option<(SourceLocation, String)> {
    context(bundle, coordinate, Anchor::Start)
}

/// Which byte of a coordinate the reader is sent to.
#[derive(Clone, Copy)]
enum Anchor {
    /// The coordinate's first byte: a written construct starts where it starts.
    Start,
    /// The first byte of the gap that lies on the line the gap ends in.
    LastLineOfGap,
}

/// The `path:line:column` of one coordinate and the whole source line holding
/// it.
///
/// The path is the display path, so a rejection names the file the caller
/// named. Line and column are one-based, counted in bytes: the language's
/// source is ASCII [FORM-3], so a byte column is a column.
fn context(
    bundle: &SourceBundle,
    coordinate: SyntaxCoordinate,
    anchor: Anchor,
) -> Option<(SourceLocation, String)> {
    let file = bundle.file(coordinate.source())?;
    let bytes = file.bytes();
    let start = usize::try_from(coordinate.start().value()).ok()?;
    let start = start.min(bytes.len());
    let start = match anchor {
        Anchor::Start => start,
        Anchor::LastLineOfGap => {
            let end = usize::try_from(coordinate.end().value())
                .ok()?
                .min(bytes.len());
            let final_line = bytes[..end]
                .iter()
                .rposition(|byte| *byte == b'\n')
                .map_or(0, |index| index.saturating_add(1));
            start.max(final_line)
        }
    };
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
        SourceLocation {
            path: file.display_path().to_owned(),
            line: u64::try_from(line).ok()?,
            column: u64::try_from(column).ok()?,
        },
        source_line,
    ))
}
