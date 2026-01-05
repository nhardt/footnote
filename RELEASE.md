# Releasing

Ultimately, Dioxus allows us to build for linux, mac, window, ios and android.

## Instructions

- cargo build --release --bin footnote-cli --features cli

## Linux


### deb

`dx bundle --release --features desktop --package-types "deb"`

Status: works

### rpm

`dx bundle --release --features desktop --package-types "rpm"`

Status: works outside of a missing release number. generated as:
Footnote-0.2.0-.x86_64.rpm

### appimage

`dx bundle --release --features desktop --package-types "appimage"`

Status: fails, need an AppImage icon
