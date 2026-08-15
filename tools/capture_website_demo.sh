#!/usr/bin/env bash
# Régénère les captures publiques du site à partir d'une base exclusivement fictive.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEMO_DIR="${1:-}"
OUT="${2:-$(mktemp -d /tmp/candilog-website-captures.XXXXXX)}"
BIN="$PROJECT_ROOT/target/debug/candilog"
WEB_OUT="$PROJECT_ROOT/website/public/screenshots"

if [[ -z "$DEMO_DIR" ]]; then
  DEMO_DIR="$($SCRIPT_DIR/create_website_demo_database.sh)"
else
  (cd "$PROJECT_ROOT" && cargo build --features capture --offline)
fi

DB="$DEMO_DIR/candilog.sqlite"
if [[ ! -f "$DB" ]]; then
  echo "Base de démonstration introuvable : $DB" >&2
  exit 1
fi

mkdir -p "$OUT" "$WEB_OUT"

capture() {
  local name="$1" route="$2" theme="$3"
  shift 3

  echo "Capture fictive : $name" >&2
  env -u WAYLAND_DISPLAY \
    DISPLAY=:0 \
    CANDILOG_DATA_DIR="$DEMO_DIR" \
    CANDILOG_CAPTURE_PATH="$OUT/$name.ppm" \
    CANDILOG_CAPTURE_ROUTE="$route" \
    CANDILOG_CAPTURE_THEME="$theme" \
    CANDILOG_CAPTURE_SIZE=large \
    "$@" \
    "$BIN"

  magick "$OUT/$name.ppm" -resize '2304x1236>' -quality 86 "$WEB_OUT/$name.webp"
}

command -v magick >/dev/null || {
  echo "ImageMagick est requis pour produire les fichiers WebP." >&2
  exit 1
}

capture dashboard dashboard light
capture candidatures candidatures light
capture cv-generator cv-generator light env CANDILOG_CAPTURE_CV_PREVIEW=1
capture statistiques statistiques light
capture lettre lettre light env CANDILOG_CAPTURE_LETTER_OUTPUT=1
capture intelligence parametres light
capture analyse-cv cv-import light
capture profil profil light
capture calendrier calendrier light

echo "$DEMO_DIR" >&2
echo "$OUT"
