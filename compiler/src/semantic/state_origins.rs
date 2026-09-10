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
    fn unlocated(self) -> Self;
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
            argument(0).merged(argument(1)).unlocated()
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
    fn make_unlocated(&mut self);
}

/// Retain a complete source bound while forgetting placement within this
/// value. Bounds are not exact effect origins or structural correspondences.
pub(crate) fn unlocated_routes<R: StateRoute + Ord>(routes: &mut Vec<R>) {
    for route in routes.iter_mut() {
        route.value_path_mut().clear();
        route.exclusions_mut().clear();
        route.make_unlocated();
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
    pub(crate) unlocated: bool,
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
    fn unlocated(self) -> Self {
        self.unlocated()
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
        self.unlocated == other.unlocated
            && self.source.root == other.source.root
            && self.source_value_fields == other.source_value_fields
    }
    fn project_source(&mut self, suffix: &[CheckedStateStep]) {
        if self.unlocated {
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
    fn make_unlocated(&mut self) {
        self.unlocated = true;
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
                        unlocated: false,
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
        unlocated_routes(&mut self.formals);
        self
    }
    pub(crate) fn lacks_exact_origins(&self) -> bool {
        self.unknown || self.formals.iter().any(|origin| origin.unlocated)
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
        if replacement.as_ref() == Some(&self.clone().projected_value(path)) {
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
            if formal.unlocated {
                mapped = mapped.unlocated();
            }
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
    pub(crate) unlocated: bool,
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
        self.unlocated == other.unlocated
            && self.parameter == other.parameter
            && self.parameter_fields == other.parameter_fields
    }
    fn project_source(&mut self, suffix: &[CheckedStateStep]) {
        if !self.unlocated {
            self.parameter_fields.extend_from_slice(suffix);
        }
    }
    fn make_unlocated(&mut self) {
        self.unlocated = true;
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
                unlocated: true,
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
