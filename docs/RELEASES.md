# Releases natives

## Revue des dépendances natives

La politique `cargo-deny` n'ignore que les avis « non maintenu » sans correctif sûr du
runtime Tauri Linux stable : GTK3 (et `proc-macro-error` via GTK3), ainsi que les crates
`rust-unic` tirées par `urlpattern` dans `tauri-utils`. Ces exceptions sont réexaminées le
30 novembre 2026 ou dès qu'une version stable de Tauri retire ces chaînes.

À l'échéance, un `cargo deny check` vert ne prouve rien : il le restera tant que la liste
n'aura pas changé. La revue consiste à rejouer le contrôle **sans** la liste d'exceptions,
comparer ce que l'arbre remonte réellement à ce que `deny.toml` déclare, retirer les avis
disparus et traiter ceux qui ont gagné une version corrigée. La commande est en tête de
`deny.toml`. La liste des licences autorisées obéit à la même règle : `cargo deny` signale
par `license-not-encountered` toute entrée que l'arbre ne contient plus, et une entrée
périmée fait croire à un contrôle qui n'a pas lieu.

Les polices embarquées (IBM Plex sous OFL 1.1, Material Symbols sous Apache-2.0) sont des
**assets**, jamais vus par `cargo-deny` : leur attribution vit dans
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md), comme celle du vendor
`pdf-extract`.

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

Le job `quality` exécute d'abord lint, tests et build frontend, puis formatage, Clippy,
tests Rust et `cargo-deny`. Les jobs de build dépendent explicitement de ce contrôle et ne
peuvent donc produire aucun paquet s'il échoue. Toutes les actions tierces sont référencées
par leur SHA complet ; les droits d'écriture sur le dépôt et l'OIDC sont réservés au seul
job `publish`.

## Plateformes et assets

| Plateforme | Runner CI | Paquets |
|---|---|---|
| macOS (Apple Silicon + Intel) | `macos-latest` (binaire universel) | `.dmg` |
| Windows | `windows-latest` | `.exe` (NSIS) |
| Ubuntu / Debian | `ubuntu-22.04` | `.deb` |
| Fedora / RHEL | même job Ubuntu (bundler Tauri) | `.rpm` |

`bundle.targets` (`src-tauri/tauri.conf.json`) énumère exactement ces cibles — `deb`, `rpm`,
`nsis`, `app`, `dmg` — et non `"all"`. Aucun **AppImage** n'est produit : le workflow ne le
publie pas, et `tauri build` sortait en erreur si `linuxdeploy` échouait, ce qui faisait
échouer le job `build` et sauter la publication entière. Une cible non publiée n'a pas à
pouvoir annuler une release.

Chaque paquet embarque `LICENSE`, `NOTICE` et `THIRD_PARTY_NOTICES.md`
(`bundle.resources`) : la PolyForm impose de transmettre ses termes et sa mention à qui
reçoit une copie du logiciel, et les polices redistribuées ont la même exigence.

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
entre les mains de l'utilisateur. Aucune mise à jour silencieuse n'est exécutée, et la
vérification de disponibilité n'a lieu que sur demande explicite depuis l'écran des
réglages.

Le contrat de nommage entre le workflow et `updater.rs` est verrouillé par un test
(`les_assets_du_workflow_sont_ceux_que_l_application_attend`) : renommer un asset ici casse
la suite de tests, et non la mise à jour du premier utilisateur.

## Chaîne de confiance

Trois garanties distinctes, dont deux seulement sont en place :

| Garantie | État | Ce qu'elle établit |
| --- | --- | --- |
| `SHA256SUMS` | **en place** | Le fichier téléchargé est intact. L'application le vérifie avant d'ouvrir un installateur ; l'utilisateur peut le refaire à la main. |
| Attestation de provenance Sigstore | **en place** | Le binaire a été construit par ce dépôt, depuis ce commit, par `release.yml`. Vérifiable par `gh attestation verify <fichier> --repo alexandrebouttierdev/candilog`. |
| Signature de code Windows et macOS | **absente** | Seule reconnue par SmartScreen et Gatekeeper. Demande un certificat commercial. |

L'attestation est produite par `actions/attest-build-provenance` dans le job `publish`, qui
exige les permissions `id-token: write` et `attestations: write`. Elle est **gratuite** et
ne demande aucun secret : le jeton OIDC du workflow suffit. Elle ne fait pas disparaître les
avertissements des systèmes d'exploitation — rien de gratuit ne le fait.

### Activer la signature de code

Cet état est **assumé** pour les premières versions, et annoncé partout où l'utilisateur
peut le rencontrer : `README`, notes de release, écran « Mises à jour ». Le jour où les
certificats sont acquis, voici ce qu'il faut, et rien de plus — le workflow ne contient
volontairement aucune branche inactive pour cela.

**macOS** — compte Apple Developer (99 $/an). `tauri-action` sait signer et notariser
nativement : ajouter au job `build` les variables d'environnement `APPLE_CERTIFICATE`
(certificat *Developer ID Application* exporté en `.p12`, encodé en base64),
`APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`
(mot de passe d'application) et `APPLE_TEAM_ID`, toutes en secrets du dépôt.

**Windows** — certificat de signature de code OV (~300 €/an) ou EV (jeton matériel, exigé
depuis 2023 pour une réputation SmartScreen immédiate). Renseigner
`bundle.windows.signCommand` dans `src-tauri/tauri.conf.json`, ou ajouter au job Windows une
étape `signtool` après `tauri-action`, avant la préparation des assets renommés. Le
certificat OV n'annule pas SmartScreen tout de suite : la réputation se construit au fil des
téléchargements.

Dans les deux cas, mettre à jour dans la même tâche le tableau ci-dessus, la section
« Avertissement éditeur inconnu » du `README`, les notes de release du workflow et la carte
« Installation maîtrisée » de l'écran des mises à jour.

## Côté site

Le site (`website/lib/data/plateformes.ts`) pointe directement sur
`…/releases/latest/download/candilog-<plateforme>-latest.<ext>` pour toujours servir la
dernière version publiée.

## Procédure de release

0. Lancer les validations de `docs/CODE_RULES.md` §20 **plus** `npm run tauri build`.
   Le workflow les rejoue avant les builds, mais ce filet de publication ne remplace pas la
   vérification locale. Un `git status --short` doit être vide après `cargo test` (types
   ts-rs à jour).
1. Monter `version` dans `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` et `package.json`
   (et `Cargo.lock` via `cargo build --manifest-path src-tauri/Cargo.toml`), et ouvrir la
   section correspondante de `CHANGELOG.md`.
2. Pousser sur `master` : le workflow se déclenche automatiquement. Un push sur `dev`
   ne déclenche aucune release.
   Alternative : lancer manuellement **Release** depuis l'onglet Actions.
3. Vérifier la release créée : tag `v<version>`, assets `-latest` et versionnés pour chaque
   plateforme, et présence de `SHA256SUMS` — sans lui, la mise à jour in-app refusera
   d'ouvrir l'installateur.
4. Installer au moins un paquet sur une machine propre et vérifier que
   `/usr/lib/Candilog/LICENSE`, `NOTICE` et `THIRD_PARTY_NOTICES.md` y figurent.
