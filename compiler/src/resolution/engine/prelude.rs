//! PRE-1 diagnostic preorder over ordinary supplied declaration records.
//!
//! Parsed records keep their ordinary syntax for checking; only diagnostics
//! expose these ordinals instead of the parser's internal source coordinates.

use super::super::{DeclarationClass, PreludeDeclarationId, PreludeDeclarationRecord};
use super::*;
use crate::source::PreludeSource;

pub(super) struct PreludeInventory {
    pub(super) records: Vec<PreludeDeclarationRecord>,
    pub(super) roles: Vec<Vec<(DeclarationClass, PreludeDeclarationId)>>,
    pub(super) builtins: Vec<PreludeDeclarationId>,
}

impl PreludeInventory {
    pub(super) fn build(
        syntax: &CanonicalSyntaxUnit<'_, '_, '_>,
        roles: &[ClassifiedRole],
    ) -> Result<Self, ResolutionCompilerFailure> {
        let sources = syntax.classified_bundle().source_bundle();
        let mut inventory = Self {
            records: Vec::new(),
            roles: vec![Vec::new(); roles.len()],
            builtins: Vec::new(),
        };
        // The built-in records retain their existing semantic identities.
        // Their diagnostic positions follow the same PRE-1 preorder as every
        // ordinary opaque, struct, enum, and function declaration.
        let mut builtin_origins = vec![None; PRELUDE_DECLARATIONS.len()];
        for phase in 0..5 {
            if phase == 1 || phase == 3 {
                for builtin in PRELUDE_DECLARATIONS {
                    let numeric = builtin.class == Some(DeclarationClass::NumericBound);
                    if numeric != (phase == 3) {
                        continue;
                    }
                    let id = inventory.push(builtin.spelling, builtin.class)?;
                    builtin_origins[usize::from(builtin.id.ordinal())] = Some(id);
                }
                continue;
            }
            for (index, role) in roles.iter().enumerate() {
                let Some(file) = sources.file(role.origin.coordinate.source()) else {
                    return Err(ResolutionCompilerFailure::InvalidRoleShape);
                };
                let record_phase = match file.prelude() {
                    Some(PreludeSource::Opaque) => 0,
                    Some(PreludeSource::Items) => 2,
                    Some(PreludeSource::Function) => 4,
                    None => continue,
                };
                if record_phase != phase {
                    continue;
                }
                let classes = match role.kind {
                    RawRoleKind::Declaration(declaration) => {
                        // FORM-8's generated unnamed region has no declaration
                        // spelling in PRE-1 and contributes no written record.
                        let coordinate = role.origin.coordinate;
                        let written = file.bytes().get(
                            coordinate.start().value() as usize..coordinate.end().value() as usize,
                        ) == Some(role.spelling.as_bytes());
                        if declaration == DeclarationRole::RegionParameter && !written {
                            continue;
                        }
                        let mut classes = declaration_classes(declaration);
                        if file.prelude() == Some(PreludeSource::Opaque) {
                            classes.retain(|class| *class != DeclarationClass::StructConstructor);
                        }
                        classes
                    }
                    RawRoleKind::DependentDeclaration(_) => Vec::new(),
                    _ => continue,
                };
                if classes.is_empty() {
                    inventory.push(&role.spelling, None)?;
                } else {
                    for class in classes {
                        let lookup = matches!(
                            class,
                            DeclarationClass::NominalType
                                | DeclarationClass::StructConstructor
                                | DeclarationClass::EnumVariant
                                | DeclarationClass::Function
                        );
                        let id = inventory.push(&role.spelling, lookup.then_some(class))?;
                        inventory.roles[index].push((class, id));
                    }
                }
            }
        }
        inventory.builtins = builtin_origins
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
        Ok(inventory)
    }

    fn push(
        &mut self,
        spelling: &str,
        class: Option<DeclarationClass>,
    ) -> Result<PreludeDeclarationId, ResolutionCompilerFailure> {
        let id = PreludeDeclarationId::from_index(self.records.len())
            .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        self.records.push(PreludeDeclarationRecord {
            id,
            spelling: spelling.to_owned(),
            class,
        });
        Ok(id)
    }
}
