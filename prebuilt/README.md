# Prebuilt libheif

The `prebuilt` feature links static [libheif](https://github.com/strukturag/libheif), [libde265](https://github.com/strukturag/libde265) and [dav1d](https://code.videolan.org/videolan/dav1d) libraries. The build script downloads the archive for the target from this fork's releases, verifies it against `SHA256SUMS` and links it.

```toml
[patch.crates-io]
libheif-sys = { git = "https://github.com/kmworks/libheif-sys", rev = "<commit>" }

[dependencies]
libheif-sys = { version = "5.3", features = ["prebuilt"] }
```

To build offline, extract an archive and point `LIBHEIF_STATIC_DIR` at it.

## Targets

| Target | vcpkg triplet |
|---|---|
| `x86_64-unknown-linux-gnu` | `x64-linux` |
| `aarch64-unknown-linux-gnu` | `arm64-linux` |
| `aarch64-apple-darwin` | `arm64-osx` |
| `x86_64-pc-windows-msvc` | `x64-windows-static-md` |

The libraries are built with [vcpkg](https://github.com/microsoft/vcpkg) at the commit in `vcpkg.commit`, with no encoders: x265, which is GPL, is deliberately left out. Linux archives are built on Ubuntu 24.04, and macOS archives target macOS 11.0.

## Releasing

Update `vcpkg.commit`, then push a `static-v<libheif version>-<revision>` tag. The `Prebuilt libraries` workflow builds every target and publishes the archives, the sources and `SHA256SUMS`. Then copy that `SHA256SUMS` here and update `ARCHIVE_TAG` in `link.rs`.

## Licenses

Everything here is under the repository's MIT license. The packaged libraries keep their own licenses, which each archive includes under `installed/<triplet>/share/<port>/copyright`, with the full LGPL and GPL texts in `licenses/`:

- libheif: LGPL-3.0-only AND MIT
- libde265: LGPL-3.0-only AND DOC
- dav1d: BSD-2-Clause AND ISC

Every release also publishes `libheif-prebuilt-sources.tar.gz`: the exact libheif and libde265 source archives vcpkg built, with the vcpkg ports and patches applied to them.
