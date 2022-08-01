// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use ::anyhow::Result;
use ::bindgen::{Bindings, Builder};
use ::cc::Build;
use ::std::{env, path::Path};

#[cfg(target_os = "windows")]
fn os_build() -> Result<()> {
    use ::std::path::PathBuf;

    let out_dir_s: String = env::var("OUT_DIR")?;
    let out_dir: &Path = Path::new(&out_dir_s);

    let libdpdk_path: String = env::var("LIBDPDK_PATH")?;

    let include_path: String = format!("{}{}", libdpdk_path, "\\include");
    let library_path: String = format!("{}{}", libdpdk_path, "\\lib");

    let libraries: Vec<&str> = vec![
        "rte_cfgfile",
        "rte_hash",
        "rte_cmdline",
        "rte_pci",
        "rte_ethdev",
        "rte_meter",
        "rte_net",
        "rte_mbuf",
        "rte_mempool",
        "rte_rcu",
        "rte_ring",
        "rte_eal",
        "rte_telemetry",
        "rte_kvargs",
    ];

    let cflags: &str = "-mavx";

    // Step 1: Now that we've compiled and installed DPDK, point cargo to the libraries.
    println!("cargo:rustc-link-search={}", library_path);

    for lib in &libraries {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    // Step 2: Generate bindings for the DPDK headers.
    let bindings: Bindings = Builder::default()
        .clang_arg(&format!("-I{}", include_path))
        .blocklist_type("rte_arp_ipv4")
        .blocklist_type("rte_arp_hdr")
        .blocklist_type("IMAGE_TLS_DIRECTORY")
        .blocklist_type("PIMAGE_TLS_DIRECTORY")
        .blocklist_type("PIMAGE_TLS_DIRECTORY64")
        .blocklist_type("IMAGE_TLS_DIRECTORY64")
        .blocklist_type("_IMAGE_TLS_DIRECTORY64")
        .clang_arg(cflags)
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate_comments(false)
        .generate()?;
    let bindings_out: PathBuf = out_dir.join("bindings.rs");
    bindings.write_to_file(bindings_out)?;

    // Step 3: Compile a stub file so Rust can access `inline` functions in the headers
    // that aren't compiled into the libraries.
    let mut builder: Build = cc::Build::new();
    builder.opt_level(3);
    builder.flag("-march=native");
    builder.file("inlined.c");
    builder.include(include_path);
    builder.compile("inlined");

    Ok(())
}

#[cfg(target_os = "linux")]
fn os_build() -> Result<()> {
    use ::std::process::Command;

    let out_dir_s = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_s);

    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    let cflags_bytes = Command::new("pkg-config")
        .args(&["--cflags", "libdpdk"])
        .output()
        .unwrap_or_else(|e| panic!("Failed pkg-config cflags: {:?}", e))
        .stdout;
    let cflags = String::from_utf8(cflags_bytes).unwrap();

    let mut header_locations = vec![];

    for flag in cflags.split(' ') {
        if flag.starts_with("-I") {
            let header_location = flag[2..].trim();
            header_locations.push(header_location);
        }
    }

    let ldflags_bytes = Command::new("pkg-config")
        .args(&["--libs", "libdpdk"])
        .output()
        .unwrap_or_else(|e| panic!("Failed pkg-config ldflags: {:?}", e))
        .stdout;
    let ldflags = String::from_utf8(ldflags_bytes).unwrap();

    let mut library_location = None;
    let mut lib_names = vec![];

    for flag in ldflags.split(' ') {
        if flag.starts_with("-L") {
            library_location = Some(&flag[2..]);
        } else if flag.starts_with("-l") {
            lib_names.push(&flag[2..]);
        }
    }

    // Link in `librte_net_mlx5` and its dependencies if desired.
    #[cfg(feature = "mlx5")]
    {
        lib_names.extend(&["rte_net_mlx5", "rte_bus_pci", "rte_bus_vdev", "rte_common_mlx5"]);
    }

    // Step 1: Now that we've compiled and installed DPDK, point cargo to the libraries.
    if let Some(location) = library_location {
        println!("cargo:rustc-link-search=native={}", location);
    }

    for lib_name in &lib_names {
        println!("cargo:rustc-link-lib=dylib={}", lib_name);
    }

    // Step 2: Generate bindings for the DPDK headers.
    let mut builder: Builder = Builder::default();
    for header_location in &header_locations {
        builder = builder.clang_arg(&format!("-I{}", header_location));
    }
    let bindings: Bindings = builder
        .blocklist_type("rte_arp_ipv4")
        .blocklist_type("rte_arp_hdr")
        .clang_arg("-mavx")
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate_comments(false)
        .generate()
        .unwrap_or_else(|e| panic!("Failed to generate bindings: {:?}", e));
    let bindings_out = out_dir.join("bindings.rs");
    bindings.write_to_file(bindings_out).expect("Failed to write bindings");

    // Step 3: Compile a stub file so Rust can access `inline` functions in the headers
    // that aren't compiled into the libraries.
    let mut builder: Build = cc::Build::new();
    builder.opt_level(3);
    builder.pic(true);
    builder.flag("-march=native");
    builder.file("inlined.c");
    for header_location in &header_locations {
        builder.include(header_location);
    }
    builder.compile("inlined");
    Ok(())
}

fn main() {
    match os_build() {
        Ok(()) => {},
        Err(e) => panic!("Failed to generate bindings: {:?}", e),
    }
}
