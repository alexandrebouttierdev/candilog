#!/usr/bin/env bash
# Génère l'image de partage 1200 × 630 à partir des vrais éléments visuels de Candilog.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBSITE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT="$WEBSITE_ROOT/app/opengraph-image.png"
LOGO="$WEBSITE_ROOT/public/brand/candilog.png"
SCREENSHOT="$WEBSITE_ROOT/public/screenshots/dashboard.webp"
DISPLAY_FONT="Cantarell-Extra-Bold"
BODY_FONT="$WEBSITE_ROOT/public/fonts/geist-variable.ttf"

command -v magick >/dev/null || {
  echo "ImageMagick est requis pour générer l'image Open Graph." >&2
  exit 1
}

magick \
  -size 1200x630 xc:'#f8f5ee' \
  -fill '#df9e47' -draw 'rectangle 0,0 1200,10' \
  -fill '#e7e1d6' -draw 'circle 1110,74 1160,74' \
  -fill '#efe9de' -draw 'circle 1150,128 1190,128' \
  -fill 'rgba(47,38,24,0.12)' -draw 'roundrectangle 560,150 1228,552 16,16' \
  -fill '#fffdf7' -stroke '#b9b1a4' -strokewidth 2 -draw 'roundrectangle 548,138 1216,540 16,16' \
  -fill '#ebe7df' -stroke none -draw 'roundrectangle 550,140 1214,182 14,14' \
  -fill '#ebe7df' -draw 'rectangle 550,166 1214,182' \
  -fill '#ca7a2f' -draw 'circle 572,161 577,161' \
  -fill '#aaa297' -draw 'circle 588,161 593,161' \
  -fill '#aaa297' -draw 'circle 604,161 609,161' \
  -font "$BODY_FONT" -pointsize 15 -fill '#66706b' -gravity northwest -annotate +845+151 'Candilog' \
  \( "$SCREENSHOT" -resize '640x343!' \) -gravity northwest -geometry +574+182 -composite \
  \( "$LOGO" -resize '48x48!' \) -gravity northwest -geometry +68+44 -composite \
  -font "$DISPLAY_FONT" -pointsize 27 -weight 700 -fill '#17201d' -gravity northwest -annotate +128+52 'Candilog' \
  -font "$BODY_FONT" -pointsize 15 -weight 700 -fill '#ca7a2f' -gravity northwest -annotate +70+147 'SUIVI DE CANDIDATURES  +  IA' \
  -font "$DISPLAY_FONT" -pointsize 50 -weight 800 -fill '#17201d' -interline-spacing -7 -gravity northwest -annotate +68+188 $'Votre recherche\nd’emploi.\nEnfin au même\nendroit.' \
  -font "$BODY_FONT" -pointsize 20 -weight 400 -fill '#5d6863' -interline-spacing 4 -gravity northwest -annotate +70+408 $'Candidatures, CV, lettres et entretiens\nréunis dans une application claire.' \
  -fill '#df9e47' -stroke none -draw 'roundrectangle 70,500 462,544 7,7' \
  -font "$BODY_FONT" -pointsize 15 -weight 700 -fill '#21170b' -gravity northwest -annotate +90+512 'Windows  ·  macOS  ·  Ubuntu  ·  Fedora' \
  -fill '#cfc8bc' -draw 'rectangle 70,584 1130,585' \
  -font "$BODY_FONT" -pointsize 13 -weight 600 -fill '#66706b' -gravity northwest -annotate +70+596 'CANDILOG' \
  -font "$BODY_FONT" -pointsize 13 -weight 400 -fill '#7a827d' -gravity northwest -annotate +974+596 'Alexandre Bouttier' \
  -strip "$OUTPUT"

echo "$OUTPUT"
