use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::{env, process};

/// Release of this fork holding the archives listed in `SHA256SUMS`.
const ARCHIVE_TAG: &str = "static-v1.23.7-1";
const CHECKSUMS: &str = include_str!("SHA256SUMS");
const MAX_ARCHIVE_SIZE: u64 = 256 * 1024 * 1024;

struct Platform {
    triplet: &'static str,
    /// In link order: libheif needs the decoders, which come after it.
    libs: [&'static str; 3],
    cpp: Option<&'static str>,
}

fn platform(target: &str) -> Option<Platform> {
    let unix = |triplet, cpp| Platform {
        triplet,
        libs: ["heif", "de265", "dav1d"],
        cpp: Some(cpp),
    };
    match target {
        "x86_64-unknown-linux-gnu" => Some(unix("x64-linux", "stdc++")),
        "aarch64-unknown-linux-gnu" => Some(unix("arm64-linux", "stdc++")),
        "aarch64-apple-darwin" => Some(unix("arm64-osx", "c++")),
        "x86_64-pc-windows-msvc" => Some(Platform {
            triplet: "x64-windows-static-md",
            libs: ["heif", "libde265", "dav1d"],
            cpp: None,
        }),
        _ => None,
    }
}

/// Links the target's prebuilt archive instead of looking for a system libheif.
pub fn link() {
    println!("cargo:rerun-if-changed=prebuilt");
    println!("cargo:rerun-if-env-changed=LIBHEIF_STATIC_DIR");

    let target = env::var("TARGET").unwrap();
    let Some(platform) = platform(&target) else {
        eprintln!("no prebuilt libheif for {target}");
        process::exit(1);
    };
    let root = match env::var_os("LIBHEIF_STATIC_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => fetch(&target, &PathBuf::from(env::var_os("OUT_DIR").unwrap())),
    };

    let lib_dir = root.join("installed").join(platform.triplet).join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    for lib in platform.libs {
        println!("cargo:rustc-link-lib=static={lib}");
    }
    if let Some(cpp) = platform.cpp {
        println!("cargo:rustc-link-lib=dylib={cpp}");
    }
}

/// Downloads and unpacks the target's archive, returning its root directory.
fn fetch(target: &str, out_dir: &Path) -> PathBuf {
    let name = format!("libheif-prebuilt-{target}");
    let root = out_dir.join(&name);
    let archive = format!("{name}.tar.gz");
    let expected = CHECKSUMS
        .lines()
        .find_map(|line| line.strip_suffix(&format!("  {archive}")))
        .unwrap_or_else(|| panic!("{archive} is not listed in SHA256SUMS"));

    let url = format!(
        "https://github.com/kmworks/libheif-sys/releases/download/{ARCHIVE_TAG}/{archive}"
    );
    let bytes = ureq::get(&url)
        .call()
        .and_then(|mut response| {
            response
                .body_mut()
                .with_config()
                .limit(MAX_ARCHIVE_SIZE)
                .read_to_vec()
        })
        .unwrap_or_else(|e| {
            panic!("downloading {url} failed: {e}; set LIBHEIF_STATIC_DIR to build offline")
        });

    let actual: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(actual, expected, "checksum mismatch for {archive}");

    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    let mut entries = tar::Archive::new(flate2::read::GzDecoder::new(&bytes[..]));
    for entry in entries.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap();
        let is_library = path
            .extension()
            .is_some_and(|extension| extension == "a" || extension == "lib");
        if is_library {
            entry.unpack_in(out_dir).unwrap();
        }
    }
    root
}
