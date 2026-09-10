//! Value-state routing, independent of storage addresses and loan permissions.
//!
//! A route can name a complete input value without expanding its owned object
//! graph. Strong replacement excludes the overwritten subvalue from that route
//! and installs the replacement's routes at the same value path. Thus a Box's
//! allocation remains distinct from the owner currently stored inside it.

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
    fn unlocated(self) -> Self;
    fn whole_transfer(self) -> Self;
}

/// The longest represented value path. An inexact selection lies somewhere
/// within that subtree; uncertainty there does not change independent fields.
#[derive(Clone, Debug)]
pub(crate) struct StateSelection {
    pub(crate) path: Vec<CheckedStateStep>,
    pub(crate) exact: bool,
}

impl StateSelection {
    pub(crate) fn read<I: StateImage>(&self, image: I) -> I {
        let selected = image.projected_value(&self.path);
        if self.exact {
            selected
        } else {
            selected.unlocated()
        }
    }

    pub(crate) fn replace<I: StateImage>(&self, image: I, replacement: I) -> I {
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
        image.replaced_value(&self.path, replacement)
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
        KernelRow::FixedVector
        | KernelRow::ArenaVector
        | KernelRow::ArenaVectorProved
        | KernelRow::HeapVector
        | KernelRow::ArenaFrame => I::fresh(),
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
        KernelRow::PlaceFront | KernelRow::PlaceBack if !copy_element => {
            argument(0).merged(argument(1)).whole_transfer()
        }
        KernelRow::TakeFront | KernelRow::TakeBack => {
            let value = argument(0);
            if copy_element {
                value.prefixed(&[CheckedStateStep::Field(0)])
            } else {
                let bound = value.unlocated();
                bound
                    .clone()
                    .prefixed(&[CheckedStateStep::Field(0)])
                    .merged(bound.prefixed(&[CheckedStateStep::Field(1)]))
            }
        }
        KernelRow::PlaceFront
        | KernelRow::PlaceBack
        | KernelRow::ArrayFromFixed
        | KernelRow::FixedFromArray => argument(0),
        // Views carry their separately checked backing-place provenance.
        KernelRow::SliceOf | KernelRow::MutSliceOf => I::fresh(),
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CheckedStateStep {
    Field(u32),
    Referent,
    Element(u64),
    VariantField { variant: u32, field: u32 },
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
            && route.precision() != StateOriginPrecision::Bound
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

pub(crate) fn project_routes<R: StateRoute + Ord>(
    routes: &[R],
    path: &[CheckedStateStep],
) -> Vec<R> {
    let mut selected = Vec::new();
    for route in routes {
        let mut route = route.clone();
        if route.value_path().starts_with(path) {
            route.value_path_mut().drain(..path.len());
        } else if let Some(suffix) = path.strip_prefix(route.value_path()) {
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
        self.union(&other);
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
    fn unlocated(self) -> Self {
        self.unlocated()
    }
    fn whole_transfer(self) -> Self {
        self.whole_transfer()
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
        if self.precision != StateOriginPrecision::Exact {
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
        }
    }
    pub(crate) fn formal_leaves(formal: DeclarationId, leaves: Vec<Vec<u32>>) -> Self {
        Self {
            unknown: false,
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
    }
    pub(crate) fn unlocated(mut self) -> Self {
        unlocated_routes(&mut self.formals, false);
        self
    }
    pub(crate) fn whole_transfer(mut self) -> Self {
        unlocated_routes(&mut self.formals, true);
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
            || self
                .formals
                .iter()
                .any(|origin| origin.precision == StateOriginPrecision::Bound)
    }
    pub(crate) fn projected(self, fields: &[u32]) -> Self {
        self.projected_value(&CheckedStateStep::fields(fields))
    }
    pub(crate) fn projected_value(mut self, path: &[CheckedStateStep]) -> Self {
        self.formals = project_routes(&self.formals, path);
        self
    }
    pub(crate) fn enum_payload(self, variant: u32, field: u32) -> Self {
        self.projected_value(&[CheckedStateStep::VariantField { variant, field }])
    }
    pub(crate) fn prefixed(mut self, prefix: &[CheckedStateStep]) -> Self {
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
        if let Some(replacement) = replacement {
            self.union(&replacement.prefixed(path));
        }
        self
    }
    pub(crate) fn instantiate(
        image: &CheckedResultStateOrigin,
        arguments: &[Option<Self>],
    ) -> Self {
        let formals = match image {
            CheckedResultStateOrigin::NoState => return Self::fresh(),
            CheckedResultStateOrigin::Unknown => return Self::unknown(),
            CheckedResultStateOrigin::Finite { formals } => formals,
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
                StateOriginPrecision::Whole => mapped.whole_transfer(),
                StateOriginPrecision::Bound => mapped.unlocated(),
            };
            for excluded in &formal.exclusions {
                exclude_routes(&mut mapped.formals, excluded);
            }
            result.union(&mapped.prefixed(&formal.result_fields));
        }
        result
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedResultStateOrigin {
    NoState,
    Finite {
        formals: Vec<CheckedResultStatePath>,
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
        if self.precision == StateOriginPrecision::Exact {
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
            exact: false,
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
    fn a_complete_summary_cannot_upgrade_incomplete_actual_contents() {
        let source = DeclarationId::from_index(0).unwrap();
        let summary = CheckedResultStateOrigin::Finite {
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
