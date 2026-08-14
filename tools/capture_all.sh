#!/usr/bin/env bash
# Produit la couverture visuelle complète de Candilog : 13 routes x 2 thèmes x 2 largeurs,
# plus une poignée d'états particuliers (modale de formulaire, inspecteur en colonne et en
# drawer, toast, écran d'erreur fatale, vue liste des candidatures). Rejouable à chaque
# étape de la refonte du design system, pour comparer les captures avant/après.
#
# Usage : tools/capture_all.sh [dossier de sortie]
#
# Valeurs acceptées par le harnais de capture (src/app/state.rs:527-595, src/main.rs:14-18) :
#   CANDILOG_CAPTURE_ROUTE   : candidatures | cv | entreprises | reseau | calendrier |
#                              statistiques | cv-generator | lettres | lettre | cv-import | profil |
#                              parametres. Toute autre valeur (dont "dashboard") retombe
#                              sur le tableau de bord — c'est le comportement voulu.
#   CANDILOG_CAPTURE_SIZE    : small (1100x700) | large (1800x1100). Absente => 1440x900.
#                              Aucune autre valeur n'est reconnue (pas de dimensions littérales).
#   CANDILOG_CAPTURE_THEME   : light bascule en thème clair. Absente => thème sombre.
#   CANDILOG_DATA_DIR        : dossier de données. Doit pointer .candilog-dev pour capturer
#                              sur des données réelles, sinon la base est vide.
#   CANDILOG_CAPTURE_DIALOG  : candidature | entreprise | contact | entretien | relance |
#                              profil | detail.
#   CANDILOG_CAPTURE_NOTIFICATION   : message affiché en toast.
#   CANDILOG_CAPTURE_FATAL_ERROR    : affiche l'écran d'erreur fatale si définie.
#   CANDILOG_CAPTURE_AI_RUNNING     : affiche l'état « opération IA en cours » si définie.
#   CANDILOG_CAPTURE_CV_PREVIEW     : charge la dernière version dans l'aperçu CV.
#   CANDILOG_CAPTURE_LETTER_OUTPUT  : peuple la lettre et l'historique d'itération.
#   CANDILOG_CAPTURE_CANDIDATE_VIEW : list bascule Candidatures en vue liste (défaut : pipeline).
#   CANDILOG_CAPTURE_CALENDAR_VIEW  : week | day.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="${1:-/tmp/candilog-captures}"
mkdir -p "$OUT"

BIN="$PROJECT_ROOT/target/release/candilog"
# Base de données de développement : sans elle, CANDILOG_DATA_DIR est absente, la base
# ouverte est vide et tous les écrans paraissent déserts sur les captures.
DATA_DIR="$PROJECT_ROOT/.candilog-dev"

# Les 13 routes couvertes : 12 valeurs explicites reconnues par le harnais, plus
# "dashboard" qui retombe sur le tableau de bord par le repli par défaut.
ROUTES=(dashboard candidatures cv entreprises reseau calendrier statistiques \
        cv-generator lettres lettre cv-import profil parametres)
SIZES=(small large)

echo "Compilation en mode release…"
(cd "$PROJECT_ROOT" && cargo build --release)

if [[ ! -x "$BIN" ]]; then
  echo "Binaire introuvable : $BIN" >&2
  exit 1
fi

# capture <route> <theme:dark|light> <taille:defaut|small|large> <nom-fichier> [VAR=valeur ...]
# Lance le binaire avec les variables d'environnement du harnais et attend qu'il écrive
# son PPM puis se ferme de lui-même (Message::CapturedForReview déclenche iced::exit()).
capture() {
  local route="$1" theme="$2" size="$3" name="$4"
  shift 4
  local -a env_args=(
    "CANDILOG_CAPTURE_PATH=$OUT/${name}.ppm"
    "CANDILOG_CAPTURE_ROUTE=$route"
    "CANDILOG_DATA_DIR=$DATA_DIR"
  )
  if [[ "$theme" == "light" ]]; then
    env_args+=("CANDILOG_CAPTURE_THEME=light")
  fi
  if [[ "$size" != "defaut" ]]; then
    env_args+=("CANDILOG_CAPTURE_SIZE=$size")
  fi
  env_args+=("$@")
  echo "Capture : $name"
  env "${env_args[@]}" "$BIN"
}

# --- 52 captures : 13 routes x 2 thèmes x 2 largeurs ---
for route in "${ROUTES[@]}"; do
  for theme in dark light; do
    for size in "${SIZES[@]}"; do
      capture "$route" "$theme" "$size" "${route}-${theme}-${size}"
    done
  done
done

# --- États particuliers (partie 2 de la tâche 18) ---

# 1. Modale de formulaire, en clair et en sombre.
capture candidatures dark defaut "dialog-candidature-dark" \
  "CANDILOG_CAPTURE_DIALOG=candidature"
capture candidatures light defaut "dialog-candidature-light" \
  "CANDILOG_CAPTURE_DIALOG=candidature"

# 2. Inspecteur en colonne (large) et en drawer (small), même dialogue.
capture candidatures dark large "dialog-detail-large" \
  "CANDILOG_CAPTURE_DIALOG=detail"
capture candidatures dark small "dialog-detail-small" \
  "CANDILOG_CAPTURE_DIALOG=detail"

# 3. Toast de notification.
capture dashboard dark defaut "notification-toast" \
  "CANDILOG_CAPTURE_NOTIFICATION=Sauvegarde effectuée"

# 4. Écran d'erreur fatale.
capture dashboard dark defaut "fatal-error" \
  "CANDILOG_CAPTURE_FATAL_ERROR=1"

# 5. Candidatures en vue liste, aux deux largeurs (escamotage des colonnes secondaires).
capture candidatures dark small "candidatures-list-small" \
  "CANDILOG_CAPTURE_CANDIDATE_VIEW=list"
capture candidatures dark large "candidatures-list-large" \
  "CANDILOG_CAPTURE_CANDIDATE_VIEW=list"

echo "Captures PPM écrites dans $OUT"

if command -v magick >/dev/null 2>&1; then
  echo "Conversion en PNG via ImageMagick…"
  for f in "$OUT"/*.ppm; do
    magick "$f" "${f%.ppm}.png"
  done
  echo "Captures PNG écrites dans $OUT"
else
  echo "ImageMagick (magick) introuvable : les captures restent au format PPM dans $OUT." >&2
fi
