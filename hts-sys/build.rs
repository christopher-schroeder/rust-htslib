// Copyright 2014 Johannes Köster.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "serde")]
use bindgen;
use cc;
use fs_utils::copy::copy_directory;
use glob::glob;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn sed_htslib_makefile(out: &PathBuf, patterns: &Vec<&str>, feature: &str) {
    for pattern in patterns {
        if Command::new("sed")
            .current_dir(out.join("htslib"))
            .arg("-i")
            .arg("-e")
            .arg(pattern)
            .arg("Makefile")
            .status()
            .unwrap()
            .success()
            != true
        {
            panic!("failed to strip {} support", feature);
        }
    }
}

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut cfg = cc::Build::new();
    cfg.warnings(false).static_flag(true).pic(true);

    if let Ok(z_inc) = env::var("DEP_Z_INCLUDE") {
        cfg.include(z_inc);
    }

    if !out.join("htslib").exists() {
        copy_directory("htslib", &out).unwrap();
    }

    let use_bzip2 = env::var("CARGO_FEATURE_BZIP2").is_ok();
    if !use_bzip2 {
        let bzip2_patterns = vec!["s/ -lbz2//", "/#define HAVE_LIBBZ2/d"];
        sed_htslib_makefile(&out, &bzip2_patterns, "bzip2");
    } else if let Ok(inc) = env::var("DEP_BZIP2_ROOT")
        .map(PathBuf::from)
        .map(|path| path.join("include"))
    {
        cfg.include(inc);
    }

    let use_lzma = env::var("CARGO_FEATURE_LZMA").is_ok();
    if !use_lzma {
        let lzma_patterns = vec!["s/ -llzma//", "/#define HAVE_LIBLZMA/d"];
        sed_htslib_makefile(&out, &lzma_patterns, "lzma");
    } else if let Ok(inc) = env::var("DEP_LZMA_INCLUDE").map(PathBuf::from) {
        cfg.include(inc);
    }

    let tool = cfg.get_compiler();
    let (cc_path, cflags_env) = (tool.path(), tool.cflags_env());
    let cc_cflags = cflags_env.to_string_lossy().replace("-O0", "");
    if Command::new("make")
        .current_dir(out.join("htslib"))
        .arg(format!("CC={}", cc_path.display()))
        .arg(format!("CFLAGS={}", cc_cflags))
        .arg("lib-static")
        .arg("-B")
        .status()
        .unwrap()
        .success()
        != true
    {
        panic!("failed to build htslib");
    }

    cfg.file("wrapper.c").compile("wrapper");

    // If bindgen is enabled, use it
    #[cfg(feature = "bindgen")]
    {
        bindgen::Builder::default()
            .header("wrapper.h")
            .generate_comments(false)
            .blacklist_function("strtold")
            .blacklist_type("max_align_t")
            .generate()
            .expect("Unable to generate bindings.")
            .write_to_file(out.join("bindings.rs"))
            .expect("Could not write bindings.");
    }

    // If no bindgen, use pre-built bindings
    #[cfg(all(not(feature = "bindgen"), target_os="macos"))]
    {
        fs::copy("osx_prebuilt_bindings.rs", out.join("bindings.rs"))
            .expect("couldn't copy prebuilt bindings");
    }

    #[cfg(all(not(feature = "bindgen"), target_os="linux"))]
    {
        fs::copy("linux_prebuilt_bindings.rs", out.join("bindings.rs"))
            .expect("couldn't copy prebuilt bindings");
    }

    let include = out.join("include");
    fs::create_dir_all(&include).unwrap();
    if include.join("htslib").exists() {
        fs::remove_dir_all(include.join("htslib")).expect("remove exist include dir");
    }
    copy_directory(out.join("htslib").join("htslib"), &include).unwrap();
    fs::copy(out.join("htslib").join("libhts.a"), out.join("libhts.a")).unwrap();

    println!("cargo:root={}", out.display());
    println!("cargo:include={}", include.display());
    println!("cargo:libdir={}", out.display());
    println!("cargo:rustc-link-lib=static=hts");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=htslib/Makefile");
    let globs = std::iter::empty()
        .chain(glob("htslib/*.[ch]").unwrap())
        .chain(glob("htslib/cram/*.[ch]").unwrap())
        .chain(glob("htslib/htslib/*.h").unwrap())
        .chain(glob("htslib/os/*.[ch]").unwrap())
        .filter_map(Result::ok);
    for htsfile in globs {
        println!("cargo:rerun-if-changed={}", htsfile.display());
    }
}
