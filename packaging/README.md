# Empaquetage

Fichiers requis par les formats de paquets Linux. La tâche d'intégration continue qui les
assemble en `.deb`, `.rpm` et AppImage est hors du périmètre courant (voir `docs/RELEASES.md`).

| Fichier | Destination |
|---|---|
| `candilog.desktop` | `/usr/share/applications/candilog.desktop` |
| `candilog.png` | `/usr/share/icons/hicolor/256x256/apps/candilog.png` |
| binaire `candilog` | `/usr/bin/candilog` |

Sans le `.desktop`, le binaire n'apparaît dans aucun menu d'applications et n'a pas d'icône
dans la barre des tâches. L'icône de **fenêtre** est, elle, embarquée dans le binaire
(`src/core/logging.rs`, `icone_application`).

## Dépendances à déclarer

- `.deb` : `libxkbcommon0`, `libwayland-client0`, `libx11-6`, `libdbus-1-3`, `libvulkan1`
- `.rpm` : `libxkbcommon`, `libwayland-client`, `libX11`, `dbus-libs`, `vulkan-loader`

## Signature de code

Windows (SmartScreen) et macOS (Gatekeeper) rejettent un binaire non signé. La signature est
traitée séparément, comme le prévoit `docs/RELEASES.md`.
