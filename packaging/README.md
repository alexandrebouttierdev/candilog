# Empaquetage

Fichiers requis par les formats de paquets Linux. La tâche d'intégration continue qui les
assemble en `.deb`, `.rpm` et AppImage est hors du périmètre courant (voir `docs/RELEASES.md`).

| Fichier | Destination |
|---|---|
| `candilog.desktop` | `/usr/share/applications/candilog.desktop` |
| `icons/hicolor/*/apps/candilog.png` | `/usr/share/icons/hicolor/*/apps/candilog.png` |
| binaire `candilog` | `/usr/bin/candilog` |

Sans le `.desktop`, le binaire n'apparaît dans aucun menu d'applications et n'a pas d'icône
dans la barre des tâches. L'icône de **fenêtre** est, elle, embarquée dans le binaire
(`src/core/logging.rs`, `icone_application`).

Le jeu d'icônes Linux est régénéré depuis le SVG de marque avec
`tools/generate_app_icons.sh`. Les PNG sont encodés en RGBA 8 bits, format accepté par les
shells Linux, et l'identifiant Wayland `candilog` correspond exactement au nom du fichier
`.desktop`.

## Dépendances à déclarer

- `.deb` : `libxkbcommon0`, `libwayland-client0`, `libx11-6`, `libdbus-1-3`, `libvulkan1`
- `.rpm` : `libxkbcommon`, `libwayland-client`, `libX11`, `dbus-libs`, `vulkan-loader`

## Signature de code

Windows (SmartScreen) et macOS (Gatekeeper) rejettent un binaire non signé. La signature est
traitée séparément, comme le prévoit `docs/RELEASES.md`.
