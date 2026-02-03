#!/bin/bash

# until https://github.com/DioxusLabs/dioxus/pull/5195 or similar merges, insert this file into the build
mkdir -p ../../target/dx/footnote-ui/release/android/app/app/src/main/res/xml
cp ./platform_build/file_paths.xml ../../target/dx/footnote-ui/release/android/app/app/src/main/res/xml/
mkdir -p ../../target/dx/footnote-ui/debug/android/app/app/src/main/res/xml
cp ./platform_build/file_paths.xml ../../target/dx/footnote-ui/debug/android/app/app/src/main/res/xml/
