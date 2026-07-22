use std::{env, process::Command};

fn main() {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"));
    let version = match Command::new(&rustc).arg("-vV").output() {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .trim()
            .replace('\n', " | "),
        Ok(output) => format!("rustc-version-command-exit={}", output.status),
        Err(error) => format!("rustc-version-command-error={error}"),
    };
    println!("cargo:rustc-env=WHITEFOOT_WITNESS_RUSTC_ID={version}");
    if let Ok(target) = env::var("TARGET") {
        println!("cargo:rustc-env=WHITEFOOT_WITNESS_TARGET={target}");
    }
    if let Ok(host) = env::var("HOST") {
        println!("cargo:rustc-env=WHITEFOOT_WITNESS_BUILD_HOST={host}");
    }
    println!("cargo:rerun-if-env-changed=RUSTC");
}
