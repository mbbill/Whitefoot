//! Value-state routing, independent of storage addresses and loan permissions.
//!
//! A route can name a complete input value without expanding its owned object
//! graph. Strong replacement excludes the overwritten subvalue from that route
//! and installs the replacement's routes at the same value path. Thus a Box's
//! allocation remains distinct from the owner currently stored inside it.

use std::collections::{BTreeMap, BTreeSet};

use crate::DeclarationId;

use super::model::CheckedStatePath;

/// Both the body checker and callable replay use the same complete kernel
/// transfer, with their own representation of the recursive bottom element.
pub(crate) trait StateImage: Clone {
    fn fresh() -> Self;
    fn unknown() -> Self;
    fn merged(self, other: Self) -> Self;
    fn prefixed(self, path: &[CheckedStateStep]) -> Self;
    fn projected_value(self, path: &[CheckedStateStep]) -> Self;
    fn replaced_value(self, path: &[CheckedStateStep], replacement: Self) -> Self;
    fn selected_bound(self) -> Self;
    fn unlocated(self) -> Self;
    fn whole_transfer(self) -> Self;
    fn run_lengths(&self) -> Option<&KnownRunLengths>;
    fn with_run_lengths(self, lengths: KnownRunLengths) -> Self;

    fn with_run_length(self, length: Option<u64>) -> Self {
        let mut lengths = self.run_lengths().cloned().unwrap_or_default();
        lengths.lengths.remove(&Vec::new());
        if let Some(length) = length {
            lengths.lengths.insert(Vec::new(), length);
        }
        self.with_run_lengths(lengths)
    }

    fn with_variant(self, variant: u32) -> Self {
        let mut lengths = self.run_lengths().cloned().unwrap_or_default();
        lengths
            .variants
            .insert(Vec::new(), BTreeSet::from([variant]));
        self.with_run_lengths(lengths)
    }
}

/// Value facts, independent of supplier identity. Composition places disjoint
/// values together; a control-flow join retains only facts common to both.
/// Known discriminants distinguish an absent variant payload from a payload
/// with unknown length. These facts do not enumerate a run's capacity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct KnownRunLengths {
    lengths: BTreeMap<Vec<CheckedStateStep>, u64>,
    variants: BTreeMap<Vec<CheckedStateStep>, BTreeSet<u32>>,
}

impl KnownRunLengths {
    pub(crate) fn root(&self) -> Option<u64> {
        self.lengths.get(&Vec::new()).copied()
    }

    pub(crate) fn paths(&self) -> impl Iterator<Item = &[CheckedStateStep]> {
        self.lengths
            .keys()
            .chain(self.variants.keys())
            .map(Vec::as_slice)
    }

    pub(crate) fn compose(&mut self, other: &Self) {
        for (path, length) in &other.lengths {
            self.lengths.insert(path.clone(), *length);
        }
        for (path, variants) in &other.variants {
            self.variants
                .entry(path.clone())
                .or_default()
                .extend(variants);
        }
    }

    pub(crate) fn join(&mut self, other: &Self) {
        let mut lengths = BTreeMap::new();
        for (path, length) in &self.lengths {
            if other.lengths.get(path) == Some(length) || other.excludes(path) {
                lengths.insert(path.clone(), *length);
            }
        }
        for (path, length) in &other.lengths {
            if self.excludes(path) {
                lengths.insert(path.clone(), *length);
            }
        }
        let mut variants = BTreeMap::new();
        for (path, left) in &self.variants {
            if let Some(right) = other.variants.get(path) {
                variants.insert(path.clone(), left.union(right).copied().collect());
            } else if other.excludes(path) {
                variants.insert(path.clone(), left.clone());
            }
        }
        for (path, right) in &other.variants {
            if self.excludes(path) {
                variants.insert(path.clone(), right.clone());
            }
        }
        self.lengths = lengths;
        self.variants = variants;
    }

    fn excludes(&self, path: &[CheckedStateStep]) -> bool {
        path.iter().enumerate().any(|(index, step)| {
            matches!(step, CheckedStateStep::VariantField { variant, .. }
                if self.variants.get(&path[..index]).is_some_and(|active| !active.contains(variant)))
        })
    }

    pub(crate) fn prefixed(self, prefix: &[CheckedStateStep]) -> Self {
        fn prefix_paths<T>(
            paths: BTreeMap<Vec<CheckedStateStep>, T>,
            prefix: &[CheckedStateStep],
        ) -> BTreeMap<Vec<CheckedStateStep>, T> {
            paths
                .into_iter()
                .map(|(path, value)| {
                    let mut selected = prefix.to_vec();
                    selected.extend(path);
                    (selected, value)
                })
                .collect()
        }
        Self {
            lengths: prefix_paths(self.lengths, prefix),
            variants: prefix_paths(self.variants, prefix),
        }
    }

    pub(crate) fn projected(self, path: &[CheckedStateStep]) -> Self {
        // A dynamic element could also select an unrepresented element.
        // Per-slot constants alone do not establish a universal element fact.
        if path.contains(&CheckedStateStep::AnyElement) {
            return Self::default();
        }
        fn project_paths<T>(
            paths: BTreeMap<Vec<CheckedStateStep>, T>,
            path: &[CheckedStateStep],
        ) -> BTreeMap<Vec<CheckedStateStep>, T> {
            paths
                .into_iter()
                .filter_map(|(stored, value)| {
                    stored
                        .strip_prefix(path)
                        .map(|suffix| (suffix.to_vec(), value))
                })
                .collect()
        }
        Self {
            lengths: project_paths(self.lengths, path),
            variants: project_paths(self.variants, path),
        }
    }

    pub(crate) fn exclude(&mut self, path: &[CheckedStateStep]) {
        self.lengths.retain(|stored, _| !stored.starts_with(path));
        self.variants.retain(|stored, _| !stored.starts_with(path));
    }

    fn outside(&self, path: &[CheckedStateStep], keep_root: bool) -> Self {
        let mut outside = self.clone();
        let retain = |stored: &Vec<CheckedStateStep>| {
            !stored.starts_with(path) || (keep_root && stored.as_slice() == path)
        };
        outside.lengths.retain(|stored, _| retain(stored));
        outside.variants.retain(|stored, _| retain(stored));
        outside
    }
}

/// The exact prefix bounds a weak update. The complete query also retains
/// selectors after a dynamic element, so reading one field does not observe
/// every sibling in the prefix. The query alone grants no exact writeback location.
#[derive(Clone, Debug)]
pub(crate) struct StateSelection {
    pub(crate) path: Vec<CheckedStateStep>,
    pub(crate) query: Vec<CheckedStateStep>,
    pub(crate) exact: bool,
    /// An unrepresented suffix has only the prefix's layout, not the final
    /// selected type's layout. Only complete queries may preserve the latter.
    pub(crate) complete: bool,
}

impl StateSelection {
    pub(crate) fn exact(path: Vec<CheckedStateStep>) -> Self {
        Self {
            query: path.clone(),
            path,
            exact: true,
            complete: true,
        }
    }

    pub(crate) fn push(&mut self, step: CheckedStateStep) {
        self.query.push(step);
        if step == CheckedStateStep::AnyElement {
            self.exact = false;
        }
        if self.exact {
            self.path.push(step);
        }
    }

    pub(crate) fn read<I: StateImage>(&self, image: I) -> I {
        let selected = image.projected_value(if self.complete {
            &self.query
        } else {
            &self.path
        });
        if self.exact {
            selected
        } else if self.complete {
            selected.selected_bound()
        } else {
            selected.unlocated()
        }
    }

    pub(crate) fn replace<I: StateImage>(&self, image: I, replacement: I) -> I {
        let surviving_lengths = (!self.exact).then(|| {
            image.run_lengths().cloned().unwrap_or_default().outside(
                &self.path,
                self.query.get(self.path.len()) == Some(&CheckedStateStep::AnyElement),
            )
        });
        let replacement = if self.exact {
            replacement
        } else {
            // No owner outside this subtree or the incoming value can enter
            // it. An unknown slot cannot strongly exclude any old supplier.
            image
                .clone()
                .projected_value(&self.path)
                .merged(replacement)
                .unlocated()
        };
        let replaced = image.replaced_value(&self.path, replacement);
        match surviving_lengths {
            Some(lengths) => replaced.with_run_lengths(lengths),
            None => replaced,
        }
    }
}

pub(crate) fn kernel_state_image<I: StateImage>(
    row: crate::KernelRow,
    copy_element: bool,
    arguments: &[I],
) -> I {
    use crate::KernelRow;
    let argument = |index| arguments.get(index).cloned().unwrap_or_else(I::unknown);
    match row {
        KernelRow::FixedVector | KernelRow::ArenaVectorProved => {
            I::fresh().with_run_length(Some(0))
        }
        KernelRow::ArenaVector | KernelRow::HeapVector => I::fresh()
            .with_run_length(Some(0))
            .prefixed(&[CheckedStateStep::VariantField {
                variant: 1,
                field: 0,
            }]),
        KernelRow::ArenaFrame => I::fresh(),
        KernelRow::HeapBox | KernelRow::ArenaBox => {
            let value = argument(1);
            value
                .clone()
                .prefixed(&[
                    CheckedStateStep::VariantField {
                        variant: 0,
                        field: 0,
                    },
                    CheckedStateStep::Referent,
                ])
                .merged(value.prefixed(&[CheckedStateStep::VariantField {
                    variant: 1,
                    field: 0,
                }]))
        }
        KernelRow::PlaceFront | KernelRow::PlaceBack => {
            let run = argument(0);
            let length = run.run_lengths().and_then(KnownRunLengths::root);
            let next_length = length.and_then(|length| length.checked_add(1));
            if copy_element {
                run.with_run_length(next_length)
            } else if row == KernelRow::PlaceBack
                && let Some(length) = length
            {
                run.replaced_value(&[CheckedStateStep::Element(length)], argument(1))
                    .with_run_length(next_length)
            } else {
                run.merged(argument(1))
                    .whole_transfer()
                    .with_run_length(next_length)
            }
        }
        KernelRow::TakeFront | KernelRow::TakeBack => {
            let value = argument(0);
            let remaining = value
                .run_lengths()
                .and_then(KnownRunLengths::root)
                .and_then(|length| length.checked_sub(1));
            if copy_element {
                value
                    .with_run_length(remaining)
                    .prefixed(&[CheckedStateStep::Field(0)])
            } else if row == KernelRow::TakeBack
                && let Some(index) = remaining
            {
                let path = [CheckedStateStep::Element(index)];
                let taken = value.clone().projected_value(&path);
                value
                    .replaced_value(&path, I::fresh())
                    .with_run_length(remaining)
                    .prefixed(&[CheckedStateStep::Field(0)])
                    .merged(taken.prefixed(&[CheckedStateStep::Field(1)]))
            } else {
                let bound = value.unlocated();
                bound
                    .clone()
                    .with_run_length(remaining)
                    .prefixed(&[CheckedStateStep::Field(0)])
                    .merged(bound.prefixed(&[CheckedStateStep::Field(1)]))
            }
        }
        KernelRow::ArrayFromFixed | KernelRow::FixedFromArray => argument(0),
        // Views carry their separately checked backing-place provenance.
        KernelRow::SliceOf | KernelRow::MutSliceOf => I::fresh(),
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CheckedStateStep {
    Field(u32),
    Referent,
    Element(u64),
    /// A typed element query, not a particular runtime address. This step
    /// can occur in a source projection; exact destination paths exclude it.
    AnyElement,
    VariantField {
        variant: u32,
        field: u32,
    },
}

impl CheckedStateStep {
    pub(crate) fn fields(fields: &[u32]) -> Vec<Self> {
        fields.iter().copied().map(Self::Field).collect()
    }
}

pub(crate) trait StateRoute: Clone {
    fn value_path(&self) -> &[CheckedStateStep];
    fn value_path_mut(&mut self) -> &mut Vec<CheckedStateStep>;
    fn exclusions(&self) -> &[Vec<CheckedStateStep>];
    fn exclusions_mut(&mut self) -> &mut Vec<Vec<CheckedStateStep>>;
    fn same_source(&self, other: &Self) -> bool;
    fn project_source(&mut self, suffix: &[CheckedStateStep]);
    fn precision(&self) -> StateOriginPrecision;
    fn precision_mut(&mut self) -> &mut StateOriginPrecision;
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum StateOriginPrecision {
    /// Destination selectors also select the corresponding source subvalue.
    Exact,
    /// A complete typed choice retains internal selector correspondence, but
    /// not an exact chosen supplier or coverage of every possible supplier.
    Selected,
    /// All source state is retained, but its positions within the value differ.
    Whole,
    /// Some source state may be retained; this is only a supplier upper bound.
    Bound,
}

/// Forget placement within a value. An intact transfer preserves complete
/// coverage; a prior cut or bound cannot acquire it. Neither unlocated form
/// claims structural correspondence for a selected descendant.
pub(crate) fn unlocated_routes<R: StateRoute + Ord>(routes: &mut Vec<R>, whole: bool) {
    for route in routes.iter_mut() {
        let precision = if whole
            && matches!(
                route.precision(),
                StateOriginPrecision::Exact | StateOriginPrecision::Whole
            )
            && route.exclusions().is_empty()
        {
            StateOriginPrecision::Whole
        } else {
            StateOriginPrecision::Bound
        };
        *route.precision_mut() = precision;
        route.value_path_mut().clear();
        route.exclusions_mut().clear();
    }
    let mut canonical = Vec::new();
    union_routes(&mut canonical, routes);
    *routes = canonical;
}

/// A dynamic choice does not locate the selected input, but retains the
/// selected value's known internal field layout. Forgetting that layout would
/// make a later key projection include an unrelated payload supplier.
pub(crate) fn bound_selected_routes<R: StateRoute + Ord>(routes: &mut Vec<R>) {
    for route in routes.iter_mut() {
        *route.precision_mut() = match route.precision() {
            StateOriginPrecision::Exact | StateOriginPrecision::Selected => {
                StateOriginPrecision::Selected
            }
            StateOriginPrecision::Whole | StateOriginPrecision::Bound => {
                StateOriginPrecision::Bound
            }
        };
    }
    let mut canonical = Vec::new();
    union_routes(&mut canonical, routes);
    *routes = canonical;
}

pub(crate) fn project_routes<R: StateRoute + Ord>(
    routes: &[R],
    path: &[CheckedStateStep],
) -> Vec<R> {
    let mut selected = Vec::new();
    for route in routes {
        let mut route = route.clone();
        if query_matches_prefix(path, route.value_path()) {
            route.value_path_mut().drain(..path.len());
        } else if route.value_path().len() <= path.len()
            && query_matches_prefix(&path[..route.value_path().len()], route.value_path())
        {
            let suffix = &path[route.value_path().len()..];
            if route
                .exclusions()
                .iter()
                .any(|excluded| suffix.starts_with(excluded))
            {
                continue;
            }
            let exclusions = route
                .exclusions()
                .iter()
                .filter_map(|excluded| excluded.strip_prefix(suffix).map(<[_]>::to_vec))
                .collect();
            route.project_source(suffix);
            route.value_path_mut().clear();
            *route.exclusions_mut() = exclusions;
        } else {
            continue;
        }
        selected.push(route);
    }
    // Selecting a subtree can make formerly distinct whole-value and child
    // routes coincide. Keep the same canonical join representation here.
    let mut canonical = Vec::new();
    union_routes(&mut canonical, &selected);
    canonical
}

fn query_matches_prefix(query: &[CheckedStateStep], path: &[CheckedStateStep]) -> bool {
    query.len() <= path.len()
        && query.iter().zip(path).all(|(query, step)| {
            query == step
                || matches!(
                    (query, step),
                    (CheckedStateStep::AnyElement, CheckedStateStep::Element(_))
                )
        })
}

pub(crate) fn exclude_routes<R: StateRoute>(routes: &mut Vec<R>, path: &[CheckedStateStep]) {
    routes.retain_mut(|route| {
        if route.value_path().starts_with(path) {
            return false;
        }
        if let Some(suffix) = path.strip_prefix(route.value_path()) {
            if route.precision() == StateOriginPrecision::Whole {
                // The cut may remove any of the unlocated source contents.
                *route.precision_mut() = StateOriginPrecision::Bound;
            }
            let suffix = suffix.to_vec();
            let exclusions = route.exclusions_mut();
            if !exclusions.iter().any(|old| suffix.starts_with(old)) {
                exclusions.retain(|old| !old.starts_with(&suffix));
                exclusions.push(suffix);
                exclusions.sort();
            }
        }
        true
    });
}

pub(crate) fn union_routes<R: StateRoute + Ord>(routes: &mut Vec<R>, added: &[R]) {
    for route in added {
        if let Some(existing) = routes.iter_mut().find(|existing| {
            existing.value_path() == route.value_path() && existing.same_source(route)
        }) {
            // A may-origin is absent after a join only where both incoming
            // images exclude it. Prefix intersections retain the narrower cut.
            let mut common = Vec::new();
            for left in existing.exclusions() {
                for right in route.exclusions() {
                    let cut = if left.starts_with(right) {
                        Some(left)
                    } else if right.starts_with(left) {
                        Some(right)
                    } else {
                        None
                    };
                    if let Some(cut) = cut
                        && !common.iter().any(|old: &Vec<_>| cut.starts_with(old))
                    {
                        common.retain(|old| !old.starts_with(cut));
                        common.push(cut.clone());
                    }
                }
            }
            common.sort();
            *existing.exclusions_mut() = common;
        } else {
            routes.push(route.clone());
        }
    }
    routes.sort();
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CheckedStateOrigins {
    pub(crate) unknown: bool,
    pub(crate) formals: Vec<CheckedStateOrigin>,
    pub(crate) run_lengths: KnownRunLengths,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CheckedStateOrigin {
    pub(crate) precision: StateOriginPrecision,
    pub(crate) value_fields: Vec<CheckedStateStep>,
    pub(crate) exclusions: Vec<Vec<CheckedStateStep>>,
    pub(crate) source_value_fields: Vec<CheckedStateStep>,
    pub(crate) source: CheckedStatePath,
}

impl StateImage for CheckedStateOrigins {
    fn fresh() -> Self {
        Self::fresh()
    }
    fn unknown() -> Self {
        Self::unknown()
    }
    fn merged(mut self, other: Self) -> Self {
        self.compose(&other);
        self
    }
    fn prefixed(self, path: &[CheckedStateStep]) -> Self {
        self.prefixed(path)
    }
    fn projected_value(self, path: &[CheckedStateStep]) -> Self {
        self.projected_value(path)
    }
    fn replaced_value(self, path: &[CheckedStateStep], replacement: Self) -> Self {
        self.replace_value_path(path, Some(replacement))
    }
    fn selected_bound(mut self) -> Self {
        bound_selected_routes(&mut self.formals);
        self
    }
    fn unlocated(self) -> Self {
        self.unlocated()
    }
    fn whole_transfer(self) -> Self {
        self.whole_transfer()
    }
    fn run_lengths(&self) -> Option<&KnownRunLengths> {
        (!self.unknown).then_some(&self.run_lengths)
    }
    fn with_run_lengths(mut self, lengths: KnownRunLengths) -> Self {
        if !self.unknown {
            self.run_lengths = lengths;
        }
        self
    }
}

impl StateRoute for CheckedStateOrigin {
    fn value_path(&self) -> &[CheckedStateStep] {
        &self.value_fields
    }
    fn value_path_mut(&mut self) -> &mut Vec<CheckedStateStep> {
        &mut self.value_fields
    }
    fn exclusions(&self) -> &[Vec<CheckedStateStep>] {
        &self.exclusions
    }
    fn exclusions_mut(&mut self) -> &mut Vec<Vec<CheckedStateStep>> {
        &mut self.exclusions
    }
    fn same_source(&self, other: &Self) -> bool {
        self.precision == other.precision
            && self.source.root == other.source.root
            && self.source_value_fields == other.source_value_fields
    }
    fn project_source(&mut self, suffix: &[CheckedStateStep]) {
        if !matches!(
            self.precision,
            StateOriginPrecision::Exact | StateOriginPrecision::Selected
        ) {
            if !suffix.is_empty() {
                self.precision = StateOriginPrecision::Bound;
            }
            return;
        }
        self.source_value_fields.extend_from_slice(suffix);
        self.source.fields = self
            .source_value_fields
            .iter()
            .map_while(|step| match step {
                CheckedStateStep::Field(field) => Some(*field),
                CheckedStateStep::Referent
                | CheckedStateStep::Element(_)
                | CheckedStateStep::AnyElement
                | CheckedStateStep::VariantField { .. } => None,
            })
            .collect();
    }
    fn precision(&self) -> StateOriginPrecision {
        self.precision
    }
    fn precision_mut(&mut self) -> &mut StateOriginPrecision {
        &mut self.precision
    }
}

impl CheckedStateOrigins {
    pub(crate) fn fresh() -> Self {
        Self::default()
    }
    pub(crate) fn unknown() -> Self {
        Self {
            unknown: true,
            formals: Vec::new(),
            run_lengths: KnownRunLengths::default(),
        }
    }
    pub(crate) fn formal_leaves(formal: DeclarationId, leaves: Vec<Vec<u32>>) -> Self {
        Self {
            unknown: false,
            run_lengths: KnownRunLengths::default(),
            formals: leaves
                .into_iter()
                .map(|fields| {
                    let value_fields = CheckedStateStep::fields(&fields);
                    CheckedStateOrigin {
                        precision: StateOriginPrecision::Exact,
                        source_value_fields: value_fields.clone(),
                        value_fields,
                        exclusions: Vec::new(),
                        source: CheckedStatePath {
                            root: formal,
                            fields,
                        },
                    }
                })
                .collect(),
        }
    }
    pub(crate) fn union(&mut self, other: &Self) {
        self.unknown |= other.unknown;
        union_routes(&mut self.formals, &other.formals);
        self.run_lengths.join(&other.run_lengths);
        if self.unknown {
            self.run_lengths = KnownRunLengths::default();
        }
    }
    pub(crate) fn compose(&mut self, other: &Self) {
        self.unknown |= other.unknown;
        union_routes(&mut self.formals, &other.formals);
        self.run_lengths.compose(&other.run_lengths);
        if self.unknown {
            self.run_lengths = KnownRunLengths::default();
        }
    }
    pub(crate) fn unlocated(mut self) -> Self {
        unlocated_routes(&mut self.formals, false);
        self.run_lengths = KnownRunLengths::default();
        self
    }
    pub(crate) fn whole_transfer(mut self) -> Self {
        unlocated_routes(&mut self.formals, true);
        self.run_lengths = KnownRunLengths::default();
        self
    }
    pub(crate) fn lacks_exact_origins(&self) -> bool {
        self.unknown
            || self
                .formals
                .iter()
                .any(|origin| origin.precision != StateOriginPrecision::Exact)
    }
    /// A declared effect on the complete actual value needs all of its
    /// sources, but does not select one stored component or release action.
    #[cfg(test)]
    pub(crate) fn lacks_whole_origins(&self) -> bool {
        self.unknown
            || self.formals.iter().any(|origin| {
                matches!(
                    origin.precision,
                    StateOriginPrecision::Selected | StateOriginPrecision::Bound
                )
            })
    }
    pub(crate) fn projected(self, fields: &[u32]) -> Self {
        self.projected_value(&CheckedStateStep::fields(fields))
    }
    pub(crate) fn projected_value(mut self, path: &[CheckedStateStep]) -> Self {
        self.formals = project_routes(&self.formals, path);
        self.run_lengths = self.run_lengths.projected(path);
        self
    }
    pub(crate) fn enum_payload(self, variant: u32, field: u32) -> Self {
        self.projected_value(&[CheckedStateStep::VariantField { variant, field }])
    }
    pub(crate) fn prefixed(mut self, prefix: &[CheckedStateStep]) -> Self {
        self.run_lengths = self.run_lengths.prefixed(prefix);
        for origin in &mut self.formals {
            let mut path = prefix.to_vec();
            path.append(&mut origin.value_fields);
            origin.value_fields = path;
        }
        self
    }
    pub(crate) fn replace_value_path(
        mut self,
        path: &[CheckedStateStep],
        replacement: Option<Self>,
    ) -> Self {
        if path.is_empty() {
            return replacement.unwrap_or_else(Self::fresh);
        }
        let selected = self.clone().projected_value(path);
        if !selected.lacks_exact_origins() && replacement.as_ref() == Some(&selected) {
            return self;
        }
        exclude_routes(&mut self.formals, path);
        self.run_lengths.exclude(path);
        if let Some(replacement) = replacement {
            self.compose(&replacement.prefixed(path));
        }
        self
    }
    pub(crate) fn instantiate(
        image: &CheckedResultStateOrigin,
        arguments: &[Option<Self>],
    ) -> Self {
        let (formals, lengths) = match image {
            CheckedResultStateOrigin::NoState => return Self::fresh(),
            CheckedResultStateOrigin::Unknown => return Self::unknown(),
            CheckedResultStateOrigin::Finite {
                formals,
                run_lengths,
            } => (formals, run_lengths),
        };
        let mut result = Self::fresh();
        for formal in formals {
            let Some(argument) = arguments.get(formal.parameter as usize) else {
                return Self::unknown();
            };
            let mut mapped = argument
                .clone()
                .unwrap_or_else(Self::fresh)
                .projected_value(&formal.parameter_fields);
            mapped = match formal.precision {
                StateOriginPrecision::Exact => mapped,
                StateOriginPrecision::Selected => mapped.selected_bound(),
                StateOriginPrecision::Whole => mapped.whole_transfer(),
                StateOriginPrecision::Bound => mapped.unlocated(),
            };
            // Owner correspondence is not a window-length postcondition.
            // A helper may preserve the former while changing the latter.
            mapped.run_lengths = KnownRunLengths::default();
            for excluded in &formal.exclusions {
                exclude_routes(&mut mapped.formals, excluded);
            }
            result.compose(&mapped.prefixed(&formal.result_fields));
        }
        result.run_lengths = lengths.clone();
        result
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedResultStateOrigin {
    NoState,
    Finite {
        formals: Vec<CheckedResultStatePath>,
        run_lengths: KnownRunLengths,
    },
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CheckedResultStatePath {
    pub(crate) precision: StateOriginPrecision,
    pub(crate) result_fields: Vec<CheckedStateStep>,
    pub(crate) parameter: u32,
    pub(crate) parameter_fields: Vec<CheckedStateStep>,
    pub(crate) exclusions: Vec<Vec<CheckedStateStep>>,
}

impl StateRoute for CheckedResultStatePath {
    fn value_path(&self) -> &[CheckedStateStep] {
        &self.result_fields
    }
    fn value_path_mut(&mut self) -> &mut Vec<CheckedStateStep> {
        &mut self.result_fields
    }
    fn exclusions(&self) -> &[Vec<CheckedStateStep>] {
        &self.exclusions
    }
    fn exclusions_mut(&mut self) -> &mut Vec<Vec<CheckedStateStep>> {
        &mut self.exclusions
    }
    fn same_source(&self, other: &Self) -> bool {
        self.precision == other.precision
            && self.parameter == other.parameter
            && self.parameter_fields == other.parameter_fields
    }
    fn project_source(&mut self, suffix: &[CheckedStateStep]) {
        if matches!(
            self.precision,
            StateOriginPrecision::Exact | StateOriginPrecision::Selected
        ) {
            self.parameter_fields.extend_from_slice(suffix);
        } else if !suffix.is_empty() {
            self.precision = StateOriginPrecision::Bound;
        }
    }
    fn precision(&self) -> StateOriginPrecision {
        self.precision
    }
    fn precision_mut(&mut self) -> &mut StateOriginPrecision {
        &mut self.precision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedBorrowedStateOrigin {
    pub(crate) parameter: u32,
    pub(crate) origin: CheckedResultStateOrigin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_lengths_compose_by_place_but_join_only_common_facts() {
        let first = CheckedStateOrigins::fresh().with_run_length(Some(1));
        let second = CheckedStateOrigins::fresh().with_run_length(Some(2));
        let mut record = first.prefixed(&[CheckedStateStep::Field(0)]);
        record.compose(&second.prefixed(&[CheckedStateStep::Field(1)]));
        assert_eq!(record.clone().projected(&[0]).run_lengths.root(), Some(1));
        assert_eq!(record.clone().projected(&[1]).run_lengths.root(), Some(2));
        let alternate = record.clone().replace_value_path(
            &[CheckedStateStep::Field(0)],
            Some(CheckedStateOrigins::fresh().with_run_length(Some(3))),
        );
        record.union(&alternate);
        assert_eq!(record.clone().projected(&[0]).run_lengths.root(), None);
        assert_eq!(record.projected(&[1]).run_lengths.root(), Some(2));
    }

    #[test]
    fn absent_variants_are_neutral_but_unknown_payload_lengths_are_not() {
        let some = CheckedStateStep::VariantField {
            variant: 1,
            field: 0,
        };
        let mut optional = CheckedStateOrigins::fresh()
            .with_run_length(Some(1))
            .prefixed(&[some])
            .with_variant(1);
        optional.union(&CheckedStateOrigins::fresh().with_variant(0));
        assert_eq!(
            optional.clone().projected_value(&[some]).run_lengths.root(),
            Some(1)
        );
        let longer = CheckedStateOrigins::fresh()
            .with_run_length(Some(2))
            .prefixed(&[some])
            .with_variant(1);
        let mut differing = optional.clone();
        differing.union(&longer);
        assert_eq!(differing.projected_value(&[some]).run_lengths.root(), None);
        optional.union(&CheckedStateOrigins::fresh().with_variant(1));
        assert_eq!(optional.projected_value(&[some]).run_lengths.root(), None);
    }

    #[test]
    fn owner_correspondence_alone_does_not_forward_a_callers_length() {
        let source = DeclarationId::from_index(0).unwrap();
        let actual =
            CheckedStateOrigins::formal_leaves(source, vec![Vec::new()]).with_run_length(Some(3));
        let summary = CheckedResultStateOrigin::Finite {
            formals: vec![CheckedResultStatePath {
                precision: StateOriginPrecision::Exact,
                result_fields: Vec::new(),
                parameter: 0,
                parameter_fields: Vec::new(),
                exclusions: Vec::new(),
            }],
            run_lengths: KnownRunLengths::default(),
        };
        let returned = CheckedStateOrigins::instantiate(&summary, &[Some(actual)]);
        assert_eq!(returned.run_lengths.root(), None);
        assert!(!returned.lacks_exact_origins());
    }

    #[test]
    fn dynamic_element_replacement_preserves_only_the_enclosing_length() {
        let child = CheckedStateOrigins::fresh().with_run_length(Some(2));
        let outer = child
            .prefixed(&[CheckedStateStep::Element(0)])
            .with_run_length(Some(1));
        let selection = StateSelection {
            path: Vec::new(),
            query: vec![CheckedStateStep::AnyElement],
            exact: false,
            complete: true,
        };
        let updated =
            selection.replace(outer, CheckedStateOrigins::fresh().with_run_length(Some(7)));
        assert_eq!(updated.run_lengths.root(), Some(1));
        assert_eq!(
            updated
                .projected_value(&[CheckedStateStep::Element(0)])
                .run_lengths
                .root(),
            None
        );
    }

    #[test]
    fn back_extraction_removes_the_taken_source_from_the_remaining_slots() {
        let first = DeclarationId::from_index(0).unwrap();
        let second = DeclarationId::from_index(1).unwrap();
        let mut value = CheckedStateOrigins::fresh().with_run_length(Some(0));
        for source in [first, second] {
            value = kernel_state_image(
                crate::KernelRow::PlaceBack,
                false,
                &[
                    value,
                    CheckedStateOrigins::formal_leaves(source, vec![Vec::new()]),
                ],
            );
        }
        let split = kernel_state_image(crate::KernelRow::TakeBack, false, &[value]);
        let taken = split.clone().projected(&[1]);
        assert_eq!(taken.formals.len(), 1);
        assert_eq!(taken.formals[0].source.root, second);
        let remaining = split.projected(&[0]);
        assert_eq!(remaining.run_lengths.root(), Some(1));
        assert_eq!(remaining.formals.len(), 1);
        assert_eq!(remaining.formals[0].source.root, first);
    }

    #[test]
    fn an_unrepresented_suffix_cannot_reuse_the_prefix_values_field_layout() {
        let payload = DeclarationId::from_index(0).unwrap();
        let image = CheckedStateOrigins::formal_leaves(payload, vec![Vec::new()])
            .prefixed(&[CheckedStateStep::Field(0), CheckedStateStep::Field(1)]);
        let selection = StateSelection {
            path: vec![CheckedStateStep::Field(0)],
            query: vec![CheckedStateStep::Field(0)],
            exact: false,
            complete: false,
        };
        let selected = selection.read(image);
        // The unknown remaining selection could reach this payload. A
        // subsequent field zero is not proven disjoint by its old position
        // at field one in the prefix value.
        let possible = selected.projected(&[0]);
        assert_eq!(possible.formals.len(), 1);
        assert_eq!(possible.formals[0].source.root, payload);
        assert_eq!(possible.formals[0].precision, StateOriginPrecision::Bound);
    }

    #[test]
    fn an_uncertain_slot_keeps_both_suppliers_but_does_not_contaminate_siblings() {
        let slots = DeclarationId::from_index(0).unwrap();
        let sibling = DeclarationId::from_index(1).unwrap();
        let incoming = DeclarationId::from_index(2).unwrap();
        let supplied = |source| CheckedStateOrigins::formal_leaves(source, vec![Vec::new()]);
        let state = supplied(slots)
            .prefixed(&[CheckedStateStep::Field(0)])
            .merged(supplied(sibling).prefixed(&[CheckedStateStep::Field(1)]));
        let selection = StateSelection {
            path: vec![CheckedStateStep::Field(0)],
            query: vec![CheckedStateStep::Field(0), CheckedStateStep::AnyElement],
            exact: false,
            complete: true,
        };
        let changed = selection.replace(state, supplied(incoming));
        assert_eq!(
            changed.clone().projected(&[1]),
            supplied(sibling),
            "an independent field still denotes its exact supplied owner"
        );
        let contents = selection.read(changed.clone());
        assert!(contents.lacks_whole_origins());
        assert_eq!(
            contents
                .formals
                .iter()
                .map(|route| route.source.root)
                .collect::<Vec<_>>(),
            vec![slots, incoming],
            "neither old nor incoming contents may be dropped from the bound"
        );
        let restored =
            changed.replace_value_path(&selection.path, Some(CheckedStateOrigins::fresh()));
        assert!(restored.clone().projected(&[0]).formals.is_empty());
        assert_eq!(restored.projected(&[1]), supplied(sibling));
    }

    #[test]
    fn whole_coverage_survives_component_transport_but_not_selection_or_partition() {
        let first = DeclarationId::from_index(0).unwrap();
        let second = DeclarationId::from_index(1).unwrap();
        let mut supplied = CheckedStateOrigins::formal_leaves(first, vec![Vec::new()]);
        supplied.union(&CheckedStateOrigins::formal_leaves(second, vec![vec![1]]));
        let complete = supplied.whole_transfer();
        assert!(!complete.lacks_whole_origins());
        assert!(complete.lacks_exact_origins());
        assert_eq!(complete.formals.len(), 2);

        let wrapped = complete.clone().prefixed(&[CheckedStateStep::Field(2)]);
        assert_eq!(wrapped.clone().projected(&[2]), complete);
        let selected =
            wrapped.projected_value(&[CheckedStateStep::Field(2), CheckedStateStep::Element(0)]);
        assert!(selected.lacks_exact_origins());
        for (before, after) in complete.formals.iter().zip(&selected.formals) {
            assert_eq!(before.source, after.source);
            assert_eq!(before.source_value_fields, after.source_value_fields);
        }

        let element_path = [CheckedStateStep::Element(0)];
        let same_bound = complete.clone().replace_value_path(
            &element_path,
            Some(complete.clone().projected_value(&element_path)),
        );
        assert!(same_bound.lacks_whole_origins());

        let cut = complete.clone().replace_value_path(
            &[CheckedStateStep::Element(0)],
            Some(CheckedStateOrigins::fresh()),
        );
        assert!(cut.clone().whole_transfer().lacks_exact_origins());
        assert_eq!(
            cut.projected_value(&[CheckedStateStep::Element(0)]),
            CheckedStateOrigins::fresh()
        );

        let taken = kernel_state_image(crate::KernelRow::TakeBack, false, &[complete]);
        assert!(taken.clone().projected(&[0]).lacks_exact_origins());
        assert!(taken.projected(&[1]).lacks_exact_origins());
    }

    #[test]
    fn typed_choice_substitution_preserves_fields_without_complete_coverage() {
        let first = DeclarationId::from_index(0).unwrap();
        let second = DeclarationId::from_index(1).unwrap();
        let supplied = |source, index| {
            CheckedStateOrigins::formal_leaves(source, vec![Vec::new()])
                .prefixed(&[CheckedStateStep::Element(index), CheckedStateStep::Field(0)])
        };
        let actual = supplied(first, 0).merged(supplied(second, 1));
        let summary = CheckedResultStateOrigin::Finite {
            run_lengths: Default::default(),
            formals: vec![CheckedResultStatePath {
                precision: StateOriginPrecision::Selected,
                result_fields: Vec::new(),
                parameter: 0,
                parameter_fields: vec![CheckedStateStep::AnyElement],
                exclusions: Vec::new(),
            }],
        };
        let result = CheckedStateOrigins::instantiate(&summary, &[Some(actual)]);
        assert_eq!(result.clone().projected(&[1]), CheckedStateOrigins::fresh());
        let file = result.projected(&[0]);
        assert_eq!(
            file.formals
                .iter()
                .map(|route| route.source.root)
                .collect::<Vec<_>>(),
            vec![first, second]
        );
        assert!(file.lacks_exact_origins());
        assert!(file.clone().whole_transfer().lacks_whole_origins());
        assert!(file.unlocated().selected_bound().lacks_exact_origins());
    }

    #[test]
    fn a_typed_choice_cannot_restore_rearranged_or_unknown_actual_layout() {
        let source = DeclarationId::from_index(0).unwrap();
        let supplied = CheckedStateOrigins::formal_leaves(source, vec![Vec::new()]);
        let summary = CheckedResultStateOrigin::Finite {
            run_lengths: Default::default(),
            formals: vec![CheckedResultStatePath {
                precision: StateOriginPrecision::Selected,
                result_fields: Vec::new(),
                parameter: 0,
                parameter_fields: vec![CheckedStateStep::AnyElement],
                exclusions: Vec::new(),
            }],
        };
        for actual in [supplied.clone().whole_transfer(), supplied.unlocated()] {
            let result = CheckedStateOrigins::instantiate(&summary, &[Some(actual)]);
            let field = result.projected(&[1]);
            assert!(field.lacks_whole_origins());
            assert_eq!(field.formals.len(), 1);
            assert_eq!(field.formals[0].source.root, source);
            assert!(field.formals[0].source_value_fields.is_empty());
        }
        let unknown =
            CheckedStateOrigins::instantiate(&summary, &[Some(CheckedStateOrigins::unknown())]);
        assert!(unknown.unknown);
    }

    #[test]
    fn a_complete_summary_cannot_upgrade_incomplete_actual_contents() {
        let source = DeclarationId::from_index(0).unwrap();
        let summary = CheckedResultStateOrigin::Finite {
            run_lengths: Default::default(),
            formals: vec![CheckedResultStatePath {
                precision: StateOriginPrecision::Whole,
                result_fields: vec![CheckedStateStep::Field(1)],
                parameter: 0,
                parameter_fields: vec![CheckedStateStep::Field(0)],
                exclusions: Vec::new(),
            }],
        };
        let exact = CheckedStateOrigins::formal_leaves(source, vec![vec![0]]);
        let complete = CheckedStateOrigins::instantiate(&summary, &[Some(exact.clone())]);
        assert!(!complete.clone().projected(&[1]).lacks_whole_origins());
        assert!(
            complete
                .projected_value(&[CheckedStateStep::Field(1), CheckedStateStep::Element(0)])
                .lacks_exact_origins()
        );

        let partial = exact.unlocated().prefixed(&[CheckedStateStep::Field(0)]);
        let result = CheckedStateOrigins::instantiate(&summary, &[Some(partial)]);
        assert!(result.projected(&[1]).lacks_exact_origins());
        let unknown =
            CheckedStateOrigins::instantiate(&summary, &[Some(CheckedStateOrigins::unknown())]);
        assert!(unknown.unknown);
    }

    #[test]
    fn unlocated_bounds_keep_suppliers_and_fresh_overrides_through_substitution() {
        let source = DeclarationId::from_index(0).unwrap();
        let bound = CheckedStateOrigins::formal_leaves(source, vec![Vec::new()]).unlocated();
        let replaced = bound.replace_value_path(
            &[CheckedStateStep::Field(0)],
            Some(CheckedStateOrigins::fresh()),
        );
        assert_eq!(
            replaced.clone().projected(&[0]),
            CheckedStateOrigins::fresh()
        );
        let sibling = replaced.projected(&[1]);
        assert!(sibling.lacks_exact_origins());
        assert_eq!(
            sibling.formals[0].source_value_fields,
            Vec::<CheckedStateStep>::new()
        );

        let summary = CheckedResultStateOrigin::Finite {
            run_lengths: Default::default(),
            formals: vec![CheckedResultStatePath {
                precision: StateOriginPrecision::Bound,
                result_fields: Vec::new(),
                parameter: 0,
                parameter_fields: vec![CheckedStateStep::Field(0)],
                exclusions: vec![vec![CheckedStateStep::Field(1)]],
            }],
        };
        let actual = CheckedStateOrigins::formal_leaves(source, vec![vec![1]]);
        let fresh = CheckedStateOrigins::instantiate(&summary, &[Some(actual)]);
        assert_eq!(fresh, CheckedStateOrigins::fresh());
        let actual = CheckedStateOrigins::formal_leaves(source, vec![vec![0]]);
        let mapped = CheckedStateOrigins::instantiate(&summary, &[Some(actual)]);
        assert_eq!(mapped.clone().projected(&[1]), CheckedStateOrigins::fresh());
        assert!(mapped.projected(&[0]).lacks_exact_origins());
        assert!(
            CheckedStateOrigins::instantiate(&summary, &[Some(CheckedStateOrigins::unknown())])
                .unknown
        );
        assert!(
            CheckedStateOrigins::instantiate(
                &CheckedResultStateOrigin::Unknown,
                &[Some(CheckedStateOrigins::fresh())]
            )
            .unknown
        );
    }
}
