# Releasing

Ultimately, Dioxus allows us to build for linux, mac, window, ios and android.

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
