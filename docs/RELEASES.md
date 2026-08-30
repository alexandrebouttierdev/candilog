# Releases natives

## Revue des dépendances natives

La politique `cargo-deny` n'ignore que les avis « non maintenu » sans correctif sûr du
runtime Tauri Linux stable : GTK3 (et `proc-macro-error` via GTK3), ainsi que les crates
`rust-unic` tirées par `urlpattern` dans `tauri-utils`. Ces exceptions sont réexaminées le
30 novembre 2026 ou dès qu'une version stable de Tauri retire ces chaînes.

La chaîne PDF, elle, a été corrigée : le vendor `pdf-extract` compile avec `lopdf 0.44`, ce
qui retire `ttf-parser` non maintenu. Une dépendance yanked ou un avis disposant d'une mise à
jour sûre reste bloquant et ne doit pas rejoindre la liste d'exceptions.

## Où sont publiés les binaires

Les releases sont publiées **sur ce dépôt**
([`alexandrebouttierdev/candilog`](https://github.com/alexandrebouttierdev/candilog)),
via le workflow [`.github/workflows/release.yml`](../.github/workflows/release.yml).

Un push sur `master` (jamais sur `dev`) déclenche le build multi-plateforme et crée une
GitHub Release **publique** lorsque le tag `v<version>` n'existe pas encore. Un
`workflow_dispatch` permet aussi un lancement manuel.

## Plateformes et assets

| Plateforme | Runner CI | Paquets |
|---|---|---|
| macOS (Apple Silicon + Intel) | `macos-latest` (binaire universel) | `.dmg` |
| Windows | `windows-latest` | `.exe` (NSIS) |
| Ubuntu / Debian | `ubuntu-22.04` | `.deb` |
| Fedora / RHEL | même job Ubuntu (bundler Tauri) | `.rpm` |

Chaque asset est publié sous deux noms :

- le nom stable `-latest` (`candilog-ubuntu-latest.deb`) : URL immuable pour le site
  (`releases/latest/download/...`) ;
- le nom versionné (`candilog-ubuntu-0.0.1.deb`) : référence immuable pour la mise à jour
  in-app.

Un asset supplémentaire, `SHA256SUMS`, porte l'empreinte de tous les fichiers de la
release. Il est **obligatoire** : l'application refuse d'ouvrir un installateur dont
l'empreinte n'y figure pas ou ne correspond pas.

Si le tag `v<version>` existe déjà, la publication est sautée : pousser sans monter la
version ne crée pas de doublon.

## Côté application

Candilog interroge l'API GitHub (`releases/latest` de `candilog`) pour comparer la version
distante à la version locale, choisit l'asset adapté au système (`.deb` ou `.rpm` selon
la famille Linux, `.exe` Windows, `.dmg` macOS), le télécharge dans le dossier
Téléchargements puis le lance avec le programme d'installation par défaut du système.

Le frontend ne désigne ni l'URL ni le nom du fichier : `settings_download_update` ne prend
aucun argument et re-résout l'asset côté Rust. Le paquet est retenu en mémoire (plafonné à
256 Mio), son empreinte SHA-256 est comparée à celle publiée dans `SHA256SUMS`, et il n'est
écrit sur disque qu'ensuite — sous un nom libre, jamais en écrasant un homonyme déjà présent
dans le dossier Téléchargements.

La mise à jour est **assistée, pas automatique** : l'installation et le redémarrage restent
entre les mains de l'utilisateur. Aucune mise à jour silencieuse n'est exécutée. La signature
de code Windows (SmartScreen) et macOS (Gatekeeper) reste à traiter séparément — l'empreinte
publiée atteste du transfert, pas de l'identité de l'éditeur.

## Côté site

Le site (`website/lib/data/plateformes.ts`) pointe directement sur
`…/releases/latest/download/candilog-<plateforme>-latest.<ext>` pour toujours servir la
dernière version publiée.

## Procédure de release

1. Monter `version` dans `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` et `package.json`
   (et `Cargo.lock` via `cargo build --manifest-path src-tauri/Cargo.toml`).
2. Pousser sur `master` : le workflow se déclenche automatiquement. Un push sur `dev`
   ne déclenche aucune release.
   Alternative : lancer manuellement **Release** depuis l'onglet Actions.
3. Vérifier la release créée : tag `v<version>`, assets `-latest` et versionnés pour chaque
   plateforme, et présence de `SHA256SUMS` — sans lui, la mise à jour in-app refusera
   d'ouvrir l'installateur.
