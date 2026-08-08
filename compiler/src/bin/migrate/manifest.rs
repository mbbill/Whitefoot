//! Which conformance cases must never be re-rendered, read from the manifest.
//!
//! Canonical rendering normalizes layout and drops everything that is not in
//! the tree, so a case whose subject IS the surface form is destroyed by being
//! migrated: `form2-neg-noncanonical-ws` compiles cleanly once its indentation
//! is normalized, and a `[FORM-4]` comment case loses the comment entirely.
//!
//! The property is a rule, not a list of names. Section 2 of the kernel
//! specification, "Canonical form", is exactly `FORM-1` through `FORM-7` (plus
//! the `LEX-1` policy rule, which no case can assert), so a case whose required
//! verdict cites a `FORM-*` rule is a case about bytes. That is read off the
//! manifest here; naming the files instead is what let this one be missed until
//! its migrated copy went green while asserting a rejection.
//!
//! Only the two fields this question needs are read. A row without an `id` is
//! an annotation row rather than a case, and a row whose `expect` states no
//! `rule` is a positive case; neither can be a surface-form reject.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The `FORM-*` case ids of each manifest consulted, keyed by manifest path.
#[derive(Default)]
pub(crate) struct SurfaceFormCases {
    manifests: HashMap<PathBuf, HashSet<String>>,
}

impl SurfaceFormCases {
    /// Whether `path` is a conformance case whose subject is the surface form.
    ///
    /// The manifest is found beside the `cases` directory a case lives in, so a
    /// corpus run cannot omit it by forgetting an argument. A path with no such
    /// manifest is an ordinary source and is migrated.
    pub(crate) fn covers(&mut self, path: &Path) -> Result<bool, String> {
        let Some(manifest) = manifest_beside(path) else {
            return Ok(false);
        };
        let Some(case) = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
        else {
            return Ok(false);
        };
        if !self.manifests.contains_key(&manifest) {
            let bytes = std::fs::read(&manifest)
                .map_err(|error| format!("cannot read {}: {error}", manifest.display()))?;
            let text = String::from_utf8(bytes)
                .map_err(|error| format!("{} is not UTF-8: {error}", manifest.display()))?;
            let ids = surface_form_ids(&text)
                .map_err(|reason| format!("{}: {reason}", manifest.display()))?;
            self.manifests.insert(manifest.clone(), ids);
        }
        Ok(self
            .manifests
            .get(&manifest)
            .is_some_and(|ids| ids.contains(&case)))
    }
}

/// The manifest governing a case file, or `None` when the path is not a case.
fn manifest_beside(path: &Path) -> Option<PathBuf> {
    let cases = path.parent()?;
    if cases.file_name()? != "cases" {
        return None;
    }
    let candidate = cases.parent()?.join("manifest.jsonl");
    candidate.is_file().then_some(candidate)
}

/// Reads the ids of every case whose required verdict cites a `FORM-*` rule.
pub(crate) fn surface_form_ids(manifest: &str) -> Result<HashSet<String>, String> {
    let mut ids = HashSet::new();
    for (ordinal, line) in manifest.lines().enumerate() {
        let row = line.trim();
        if row.is_empty() || row.starts_with('#') {
            continue;
        }
        let Some((id, _)) = string_field(row, "id", 0)? else {
            // An annotation row: it covers a rule by policy and asserts no
            // source at all, so it has no bytes to render.
            continue;
        };
        let Some(expect) = row.find("\"expect\"") else {
            return Err(format!("line {} states a case with no expect", ordinal + 1));
        };
        if let Some((rule, _)) = string_field(row, "rule", expect)?
            && rule.starts_with("FORM-")
        {
            ids.insert(id);
        }
    }
    Ok(ids)
}

/// Reads the string value of `key` at or after `from`, if the row states one.
///
/// The complete key is matched, so the `rules` array cannot be mistaken for the
/// `rule` field. A value carrying an escape is refused rather than guessed at:
/// no case id or rule id has one, and silently mis-reading either would
/// under-exclude, which is the direction that destroys a case.
fn string_field(row: &str, key: &str, from: usize) -> Result<Option<(String, usize)>, String> {
    let needle = format!("\"{key}\"");
    let mut cursor = from;
    while let Some(offset) = row.get(cursor..).and_then(|rest| rest.find(&needle)) {
        let after = cursor + offset + needle.len();
        cursor = after;
        let rest = row.get(after..).unwrap_or_default().trim_start();
        let Some(rest) = rest.strip_prefix(':') else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('"') else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            return Err(format!("unterminated {key} value"));
        };
        let value = &rest[..end];
        if value.contains('\\') {
            return Err(format!("{key} value carries an escape: {value}"));
        }
        return Ok(Some((value.to_owned(), after)));
    }
    Ok(None)
}
