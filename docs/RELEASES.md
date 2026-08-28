# Releases natives

Les releases sont publiées sur le dépôt public GitHub
[`alexandrebouttierdev/candilog-releases`](https://github.com/alexandrebouttierdev/candilog-releases),
**distinct** du dépôt source `alexandrebouttierdev/candilog`. Le dépôt source ne contient que
le code et sa CI ; le dépôt des releases héberge le workflow de build et les binaires publiés.

## Déclenchement : un push sur le dépôt source

Le workflow [`.github/workflows/release-dispatch.yml`](../.github/workflows/release-dispatch.yml)
du dépôt source s'exécute à chaque push sur `master` (jamais sur `dev`). Il lit la version de
`src-tauri/Cargo.toml`, vérifie que la release `v<version>` n'existe pas encore sur
`candilog-releases` et, le cas échéant, envoie un événement `repository_dispatch` (type
`release-build`) à ce dépôt. Le workflow de build n'est donc **pas** hébergé ici : il tourne
dans `candilog-releases` et construit le commit exact qui vient d'être poussé.

Cette indirection exige un jeton avec le périmètre `repo` sur `candilog-releases`, stocké dans
les secrets du dépôt source sous le nom `CANDILOG_RELEASES_TOKEN`. Il sert **uniquement** à
déclencher le workflow distant : la publication des releases utilise le `GITHUB_TOKEN` du
dépôt des releases, qui possède les droits d'écriture sur ses propres releases.

## Workflow de `candilog-releases`

Le workflow [`.github/workflows/release.yml`](https://github.com/alexandrebouttierdev/candilog-releases/blob/main/.github/workflows/release.yml)
du dépôt des releases s'exécute sur l'événement `repository_dispatch` (ou manuellement via
`workflow_dispatch`, avec une référence source au choix). Il doit désormais construire
**l'application Tauri** (`npm ci` à la racine, puis `cargo tauri build` / `src-tauri`),
et non plus le crate Iced historique.

Les assets publiés conservent les noms attendus par le vérificateur de mises à jour :

| Plateforme | Paquets |
|---|---|
| Ubuntu/Debian | `.deb` |
| Fedora/RHEL | `.rpm` |
| macOS | `.dmg` |
| Windows | `.exe` |

Chaque asset est publié sous deux noms :

- le nom stable `-latest` (`candilog-ubuntu-latest.deb`) : URL immuable de la landing page
  (`releases/latest/download/...`) ;
- le nom versionné (`candilog-ubuntu-0.3.0.deb`) : référence immuable de la mise à jour,
  retrouvé par l'application selon l'extension de sa plateforme.

Si le tag `v<version>` existe déjà, la publication est sautée : pousser sans monter la version
ne crée pas de doublon et ne relance même pas le workflow distant.

Tant que le workflow distant n'est pas aligné sur Tauri, un push sur `master` déclenche encore
un build Iced qui échouera (le crate racine n'existe plus).

## Côté application

Candilog interroge l'API GitHub (`releases/latest` de `candilog-releases`) pour comparer la
version distante à la version locale, choisit l'asset adapté au système (`.deb` ou `.rpm`
selon la famille Linux, `.exe` Windows, `.dmg` macOS), le télécharge dans le dossier
Téléchargements puis le lance avec le programme d'installation par défaut du système.

La mise à jour est **assistée, pas automatique** : l'installation et le redémarrage restent
entre les mains de l'utilisateur. Aucune mise à jour silencieuse n'est exécutée. La signature
de code Windows (SmartScreen) et macOS (Gatekeeper) reste à traiter dans le dépôt des releases.

## Procédure de release

1. Monter `version` dans `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` et `package.json`
   (et `Cargo.lock` via `cargo build --manifest-path src-tauri/Cargo.toml`).
2. Pousser sur `master` : le workflow distant se déclenche automatiquement. Un push sur `dev`
   ne déclenche aucune release.
   Alternative : lancer manuellement `Build & Release` depuis l'onglet Actions de
   `candilog-releases`.
3. Vérifier la release créée sur `candilog-releases` : tag `v<version>`, assets `-latest` et
   versionnés pour chaque plateforme.
