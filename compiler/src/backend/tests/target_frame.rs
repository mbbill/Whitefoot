use crate::backend::qualification::{SystemTarget, qualify_program};
use crate::backend::target::{
    TargetAggregateLayout, TargetFramePlan, TargetFrameSlot, TargetLayout, TargetLayoutFailure,
    TargetObject, TargetStorageType, plan_target_frame, validate_static_storage,
};

use super::system::with_ir;

const FRAME_CONTEXT: &[u8] = br#"command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

fn plan(
    slots: &[TargetFrameSlot],
    address_index_max: Option<u64>,
) -> Result<TargetFramePlan, TargetLayoutFailure> {
    with_ir(FRAME_CONTEXT, |program| {
        let host = TargetLayout::host().expect("the frame test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification =
            qualify_program(system_target, program).expect("the frame fixture must qualify");
        let target = address_index_max
            .map(|maximum| host.with_address_index_max_for_test(maximum))
            .unwrap_or(host);
        plan_target_frame(target, &qualification, program, slots)
    })
}

fn validate_static(
    ty: &TargetStorageType,
    address_index_max: u64,
) -> Result<TargetAggregateLayout, TargetLayoutFailure> {
    with_ir(FRAME_CONTEXT, |program| {
        let host = TargetLayout::host().expect("the static-storage test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification = qualify_program(system_target, program)
            .expect("the static-storage fixture must qualify");
        let target = host.with_address_index_max_for_test(address_index_max);
        validate_static_storage(target, &qualification, program, ty)
    })
}

#[test]
fn i64_slot_after_i8_has_explicit_seven_byte_padding() {
    let slots = [
        TargetFrameSlot::natural(TargetStorageType::integer(8)),
        TargetFrameSlot::natural(TargetStorageType::integer(64)),
    ];
    let frame = plan(&slots, None).expect("the frame must be representable");

    let byte = frame.logical_field(0).expect("the byte slot must exist");
    assert_eq!(byte.physical_index(), 0);
    assert_eq!(byte.offset(), 0);
    let word = frame.logical_field(1).expect("the word slot must exist");
    assert_eq!(word.physical_index(), 2);
    assert_eq!(word.offset(), 8);
    assert_eq!(
        frame.physical_fields(),
        &[
            TargetStorageType::integer(8),
            TargetStorageType::bytes(7),
            TargetStorageType::integer(64),
        ]
    );
    assert_eq!(frame.layout().size(), 16);
    assert_eq!(frame.layout().align(), 8);
}

#[test]
fn requested_alignment_adds_tail_padding_to_byte_array_slot() {
    let slots = [TargetFrameSlot::aligned(TargetStorageType::bytes(3), 8)];
    let frame = plan(&slots, None).expect("the frame must be representable");

    let bytes = frame
        .logical_field(0)
        .expect("the byte-array slot must exist");
    assert_eq!(bytes.physical_index(), 0);
    assert_eq!(bytes.offset(), 0);
    assert_eq!(
        frame.physical_fields(),
        &[TargetStorageType::bytes(3), TargetStorageType::bytes(5),]
    );
    assert_eq!(frame.layout().size(), 8);
    assert_eq!(frame.layout().align(), 8);
}

#[test]
fn complete_frame_must_fit_the_selected_target_address_domain() {
    let slots = [
        TargetFrameSlot::natural(TargetStorageType::integer(8)),
        TargetFrameSlot::natural(TargetStorageType::integer(64)),
    ];

    assert_eq!(
        plan(&slots, Some(15)),
        Err(TargetLayoutFailure::Unrepresentable(
            TargetObject::StackFrame
        ))
    );
}

#[test]
fn scalar_static_storage_must_fit_the_selected_target_address_domain() {
    let scalar = TargetStorageType::integer(64);

    assert_eq!(
        validate_static(&scalar, 7),
        Err(TargetLayoutFailure::Unrepresentable(TargetObject::Static))
    );
    let boundary = validate_static(&scalar, 8).expect("the complete scalar fits at the boundary");
    assert_eq!(boundary.size(), 8);
    assert_eq!(boundary.align(), 8);
}

#[test]
fn pointer_static_storage_must_fit_the_selected_target_address_domain() {
    let pointer = TargetStorageType::pointer();

    assert_eq!(
        validate_static(&pointer, 7),
        Err(TargetLayoutFailure::Unrepresentable(TargetObject::Static))
    );
    let boundary = validate_static(&pointer, 8).expect("the complete pointer fits at the boundary");
    assert_eq!(boundary.size(), 8);
    assert_eq!(boundary.align(), 8);
}
