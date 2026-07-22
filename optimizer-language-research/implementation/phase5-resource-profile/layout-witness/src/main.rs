#![forbid(unsafe_code)]

use std::process::ExitCode;

use whitefoot_resource_layout_witness::{
    ALLOCATOR_ASSUMPTION, BuildIdentity, MINIMUM_PHYSICAL_MEMORY_BYTES,
    MODELED_PROCESS_TARGET_BYTES, PRIVATE_FRONTEND_RECORDS_REQUIRING_IN_CRATE_ASSERTIONS,
    RECORD_CHARGES, SUPERVISED_RSS_CEILING_BYTES, SUPERVISOR_OBLIGATIONS, observed_public_layouts,
};

fn main() -> ExitCode {
    let build = BuildIdentity::embedded();
    println!("status=non-authoritative-observation-only");
    println!("build.target={}", build.target);
    println!("build.host={}", build.build_host);
    println!("build.rustc={}", build.rustc);
    println!("assumption.allocator={ALLOCATOR_ASSUMPTION}");
    println!("assumption.minimum_physical_memory_bytes={MINIMUM_PHYSICAL_MEMORY_BYTES}");
    println!("assumption.rss_ceiling_bytes={SUPERVISED_RSS_CEILING_BYTES}");
    println!("assumption.modeled_process_target_bytes={MODELED_PROCESS_TARGET_BYTES}");

    if let Err(error) = build.validate_supported_build() {
        eprintln!("unsupported_build={error:?}");
        return ExitCode::from(2);
    }

    for layout in observed_public_layouts() {
        let charge = layout.family.charge();
        println!(
            "layout.type={} family={} size={} alignment={} stride_ceiling={} alignment_ceiling={} result=fit",
            layout.production_type,
            charge.name,
            layout.size_bytes,
            layout.alignment_bytes,
            charge.stride_bytes,
            charge.maximum_alignment
        );
    }
    for (name, family) in PRIVATE_FRONTEND_RECORDS_REQUIRING_IN_CRATE_ASSERTIONS {
        println!(
            "layout.type={name} family={} result=pending-in-crate-assertion",
            family.name()
        );
    }
    for charge in RECORD_CHARGES.iter().filter(|charge| charge.future_record) {
        println!(
            "layout.type=absent family={} stride_ceiling={} alignment_ceiling={} result=unmeasured-future-charge",
            charge.name, charge.stride_bytes, charge.maximum_alignment
        );
    }
    for obligation in SUPERVISOR_OBLIGATIONS {
        println!("supervisor.pending={obligation}");
    }
    ExitCode::SUCCESS
}
