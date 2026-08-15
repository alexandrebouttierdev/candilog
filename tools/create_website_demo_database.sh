#!/usr/bin/env bash
# Crée une base Candilog réservée aux captures publiques du site.
# Toutes les personnes, entreprises, URL et coordonnées sont fictives.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SEED="$SCRIPT_DIR/fixtures/website_demo_data.sql"

if [[ $# -gt 1 ]]; then
  echo "Usage : $0 [dossier-vide]" >&2
  exit 2
fi

if [[ $# -eq 1 ]]; then
  DEMO_DIR="$1"
  mkdir -p "$DEMO_DIR"
else
  DEMO_DIR="$(mktemp -d /tmp/candilog-website-demo.XXXXXX)"
fi

DB="$DEMO_DIR/candilog.sqlite"
if [[ -e "$DB" ]]; then
  echo "Refus de remplacer une base existante : $DB" >&2
  exit 1
fi

command -v sqlite3 >/dev/null || {
  echo "sqlite3 est requis pour créer la base de démonstration." >&2
  exit 1
}

echo "Compilation du harnais de capture…" >&2
(cd "$PROJECT_ROOT" && cargo build --features capture --offline)

echo "Création du schéma isolé…" >&2
env -u WAYLAND_DISPLAY \
  DISPLAY=:0 \
  CANDILOG_DATA_DIR="$DEMO_DIR" \
  CANDILOG_CAPTURE_PATH="$DEMO_DIR/bootstrap.ppm" \
  CANDILOG_CAPTURE_ROUTE=dashboard \
  CANDILOG_CAPTURE_THEME=dark \
  CANDILOG_CAPTURE_SIZE=small \
  "$PROJECT_ROOT/target/debug/candilog"

echo "Injection des données entièrement fictives…" >&2
sqlite3 "$DB" < "$SEED"

echo "$DEMO_DIR"
