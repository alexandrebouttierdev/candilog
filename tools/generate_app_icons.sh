#!/usr/bin/env bash
# Régénère les icônes natives depuis le SVG de marque Candilog.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SOURCE="$PROJECT_ROOT/assets/icons/candilog-icon-v2.svg"
ASSETS="$PROJECT_ROOT/assets/icons"
PACKAGING="$PROJECT_ROOT/packaging"
SIZES=(16 32 48 64 128 256 512)

command -v magick >/dev/null 2>&1 || {
  echo "ImageMagick (magick) est requis pour générer les icônes." >&2
  exit 1
}

for size in "${SIZES[@]}"; do
  asset="$ASSETS/candilog-icon-${size}.png"
  linux_dir="$PACKAGING/icons/hicolor/${size}x${size}/apps"
  mkdir -p "$linux_dir"
  magick -background none "$SOURCE" -resize "${size}x${size}" -depth 8 "PNG32:$asset"
  cp "$asset" "$linux_dir/candilog.png"
done

cp "$ASSETS/candilog-icon-256.png" "$PACKAGING/candilog.png"
magick \
  "$ASSETS/candilog-icon-16.png" \
  "$ASSETS/candilog-icon-32.png" \
  "$ASSETS/candilog-icon-48.png" \
  "$ASSETS/candilog-icon-64.png" \
  "$ASSETS/candilog-icon-128.png" \
  "$ASSETS/candilog-icon-256.png" \
  "$PACKAGING/candilog.ico"

echo "Icônes Candilog régénérées (PNG Linux et ICO Windows)."
