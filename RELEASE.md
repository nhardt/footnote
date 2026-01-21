# Releasing

Ultimately, Dioxus allows us to build for linux, mac, window, ios and android.

## Trigger the automated github process

The release cycle looks like:

- update version in Cargo.toml
`git commit -m 'version X.Y.Z'`
- do all work for X.Y.Z
`git tag vX.Y.Z`
`git push origin vX.Y.Z` # triggers build and release
- update version in Cargo.toml to X.Y.Z+1
`git commit -m 'version X.Y.Z+1'`

## Instructions

Verify all tests pass:
`cargo test --all-features`

### CLI

`cargo build --release --bin footnote-cli --features cli`

### Linux

#### deb

`dx bundle --release --features desktop --package-types "deb"`

Status: works

#### rpm

`dx bundle --release --features desktop --package-types "rpm"`

Status: works outside of a missing release number. generated as:
Footnote-0.2.0-.x86_64.rpm

#### appimage

`dx bundle --release --features desktop --package-types "appimage"`

Status: fails, need an AppImage icon

### Windows

The two hurdles two windows were getting CMake installed (or recognized) and creating
an icon file. For the icon, a 256x256 png named icons/icon.ico worked.

rustup provided some windows tools but not enough to complete the build. what is
known to work:

1. get vscode
1. install visual studio community edition
1. in Studio: Tools > Get Tools and Features > Desktop development with C++
1. install CMake from their website
1. ssh-keygen.exe
1. (reboot maybe. it worked but not sure if it was required)
1. install rust via rust website
1. cargo install cargo-binstall
1. cargo binstall dioxus-cli
1. rustup instructions from dioxus website
1. dx bundle -r --target windows

### Android

At this moment, Dioxus does not support adding file_paths.xml to the build. It
can be added before the build by running ./platform_build/dx_prebuild.sh.
