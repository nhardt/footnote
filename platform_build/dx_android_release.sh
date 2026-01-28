#!/bin/bash
set -e

# Until https://github.com/DioxusLabs/dioxus/pull/5195 or similar merges
echo "Building Android bundle with version management..."

mkdir -p ./target/dx/footnote/release/android/app/app/src/main/res/xml
cp ./platform_build/file_paths.xml ./target/dx/footnote/release/android/app/app/src/main/res/xml/

dx build --platform android --release --target aarch64-linux-android

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<< "$VERSION"
VERSION_CODE=$((MAJOR * 10000 + MINOR * 100 + PATCH))

echo "Version: $VERSION (code: $VERSION_CODE)"

GRADLE_FILE="./target/dx/footnote/release/android/app/app/build.gradle.kts"
sed -i.bak "s/versionCode = .*/versionCode = $VERSION_CODE/" "$GRADLE_FILE"
sed -i.bak "s/versionName = .*/versionName = \"$VERSION\"/" "$GRADLE_FILE"
sed -i.bak "s/minSdk = .*/minSdk = 24/" "$GRADLE_FILE"
sed -i.bak "s/targetSdk = .*/targetSdk = 35/" "$GRADLE_FILE"
sed -i.bak "s/compileSdk = .*/compileSdk = 35/" "$GRADLE_FILE"

cd target/dx/footnote/release/android/app
./gradlew bundleRelease
cd -

OUTPUT_DIR="target/dx/footnote/release/android/app/app/build/outputs/bundle/release/"
OUTPUT_FILE="${OUTPUT_DIR}/Footnote-${VERSION}.aab"
cp ./target/dx/footnote/release/android/app/app/build/outputs/bundle/release/app-release.aab \
    $OUTPUT_FILE

jarsigner -sigalg SHA256withRSA -digestalg SHA-256 -keystore .private/upload-keystore.jks \
    $OUTPUT_FILE upload

echo "Bundle created: $OUTPUT_DIR/Footnote-${VERSION}.aab"
