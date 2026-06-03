#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <target-triple> <binary-path> <package-dir>" >&2
  exit 2
fi

target_triple="$1"
binary_path="$2"
package_dir="$3"
app_name="RTVC.app"
bundle_id="one.teki.rtvc"
bundle_version="$(awk -F'"' '/^version =/ { print $2; exit }' Cargo.toml)"

if [[ ! -f "$binary_path" ]]; then
  echo "missing binary: $binary_path" >&2
  exit 1
fi

mkdir -p "$package_dir"
rm -rf "$package_dir/$app_name"
mkdir -p "$package_dir/$app_name/Contents/MacOS"
mkdir -p "$package_dir/$app_name/Contents/Resources"

cp "$binary_path" "$package_dir/$app_name/Contents/MacOS/rtvc"
chmod +x "$package_dir/$app_name/Contents/MacOS/rtvc"

# The native app searches for these directories in the current working
# directory and beside the executable. Finder launches do not set the extracted
# archive as cwd, so keep runtime assets beside Contents/MacOS/rtvc.
cp -R roms "$package_dir/$app_name/Contents/MacOS/roms"
cp -R progs "$package_dir/$app_name/Contents/MacOS/progs"
cp -R "$package_dir/web" "$package_dir/$app_name/Contents/MacOS/web"
cp README.md "$package_dir/"
cp LICENSE "$package_dir/"

cat > "$package_dir/$app_name/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>RTVC</string>
  <key>CFBundleExecutable</key>
  <string>rtvc</string>
  <key>CFBundleIdentifier</key>
  <string>${bundle_id}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>RTVC</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${bundle_version}</string>
  <key>CFBundleSupportedPlatforms</key>
  <array>
    <string>MacOSX</string>
  </array>
  <key>CFBundleVersion</key>
  <string>${GITHUB_RUN_NUMBER:-0}</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

codesign --force --deep --options runtime --sign - "$package_dir/$app_name"
codesign --verify --deep --strict --verbose=2 "$package_dir/$app_name"
