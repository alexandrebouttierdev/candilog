#!/usr/bin/env bash
# Génère l'image de partage 1200 × 630 à partir des vrais éléments visuels de Candilog.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBSITE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT="$WEBSITE_ROOT/app/opengraph-image.png"
LOGO="$WEBSITE_ROOT/public/brand/candilog.png"
SCREENSHOT="$WEBSITE_ROOT/public/screenshots/dashboard.webp"
DISPLAY_FONT="$WEBSITE_ROOT/public/fonts/geist-variable.ttf"
BODY_FONT="$WEBSITE_ROOT/public/fonts/geist-variable.ttf"

command -v magick >/dev/null || {
  echo "ImageMagick est requis pour générer l'image Open Graph." >&2
  exit 1
}

magick \
  -size 1200x630 xc:'#101114' \
  -fill '#315fc9' -stroke none -draw 'rectangle 0,0 10,630' \
  \( "$LOGO" -resize '42x42!' \) -gravity northwest -geometry +62+48 -composite \
  -stroke none \
  -font "$DISPLAY_FONT" -pointsize 25 -weight 680 -fill '#f1f2f4' -gravity northwest -annotate +120+55 'Candilog' \
  -font "$BODY_FONT" -pointsize 13 -weight 620 -fill '#7ba2ff' -kerning 2.2 -gravity northwest -annotate +64+148 'APPLICATION DE BUREAU' \
  -font "$DISPLAY_FONT" -pointsize 58 -weight 690 -fill '#f1f2f4' -interline-spacing -7 -gravity northwest -annotate +60+188 $'Votre recherche.\nUn seul espace.' \
  -font "$BODY_FONT" -pointsize 20 -weight 400 -fill '#a8acb4' -interline-spacing 5 -gravity northwest -annotate +64+340 $'Offres, contacts et documents.\nRendez-vous, dans un dossier privé.' \
  -fill '#315fc9' -draw 'roundrectangle 64,452 257,498 23,23' \
  -font "$BODY_FONT" -pointsize 15 -weight 650 -fill '#ffffff' -gravity northwest -annotate +92+465 'Installer Candilog' \
  -fill '#2b2e34' -draw 'rectangle 64,570 1138,571' \
  -font "$BODY_FONT" -pointsize 13 -weight 520 -fill '#8e929b' -gravity northwest -annotate +64+584 'WINDOWS  ·  macOS  ·  LINUX' \
  -fill '#1b1d22' -stroke '#343841' -strokewidth 2 -draw 'roundrectangle 548,126 1248,608 18,18' \
  \( "$SCREENSHOT" -resize '820x461!' \) -gravity northwest -geometry +566+145 -composite \
  -fill '#7ba2ff' -stroke none -draw 'circle 1120,64 1126,64' \
  -font "$BODY_FONT" -pointsize 12 -weight 550 -fill '#a8acb4' -gravity northwest -annotate +957+57 'DONNÉES LOCALES' \
  -strip "$OUTPUT"

echo "$OUTPUT"
