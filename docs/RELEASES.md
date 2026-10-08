# Releases natives

## Revue des dépendances natives

La politique `cargo-deny` n'ignore que les avis « non maintenu » sans correctif sûr du
runtime Tauri Linux stable : `proc-macro-error` via les liaisons GTK3, et les crates
`rust-unic` tirées par `urlpattern` dans `tauri-utils`. Ces exceptions sont réexaminées le
31 mai 2027 ou dès qu'une version stable de Tauri retire ces chaînes.

**Revue du 4 octobre 2026.** Elle a servi à quelque chose, et c'est l'argument pour la
refaire. Deux constats :

- les dix avis des liaisons GTK3 elles-mêmes (`RUSTSEC-2024-0411` à `-0420` : `gtk`,
  `gdk`, `atk`, `gtk-sys`…) ont été **retirés** de la base RustSec le 14 août 2026. Les
  crates sont toujours dans l'arbre, mais plus les avis : `cargo deny` les signalait en
  `advisory-not-detected`, et les garder faisait exactement croire à un contrôle qui
  n'avait plus lieu. Ils sont retirés de `deny.toml` ;
- `rustls` 0.23.43 portait `RUSTSEC-2026-0285` (gravité 5,3), une acceptation de messages
  de handshake TLS 1.3 au mauvais niveau de chiffrement. C'est la pile TLS de **tous** les
  appels sortants — fournisseurs IA et vérification de mise à jour. Un correctif existait
  (0.23.45) : l'avis était donc bloquant par construction, et `cargo deny check` échouait,
  ce qui interdisait toute publication puisque le job `quality` l'exécute.

L'avis `RUSTSEC-2024-0429` (`glib`, *unsound*) reste signalé en avertissement, sans mise à
jour disponible sur la lignée GTK3 ; il n'est pas bloquant et n'a pas à rejoindre la liste.

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
tests Rust, `cargo-deny` et le scénario de bout en bout des documents en mode **synthétique**
(`CANDILOG_E2E=1 CANDILOG_E2E_SYNTHETIC=1`, sans appel au fournisseur IA ni cache : le rejeu
lirait `test-output/`, que le dépôt ne versionne pas). Les jobs de build dépendent explicitement de ce contrôle et ne
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
| Arch Linux | job `arch`, conteneur `archlinux:base-devel` | `.pkg.tar.zst` |

`bundle.targets` (`src-tauri/tauri.conf.json`) énumère exactement ces cibles — `deb`, `rpm`,
`nsis`, `app`, `dmg` — et non `"all"`. Aucun **AppImage** n'est produit : le workflow ne le
publie pas, et `tauri build` sortait en erreur si `linuxdeploy` échouait, ce qui faisait
échouer le job `build` et sauter la publication entière. Une cible non publiée n'a pas à
pouvoir annuler une release.

Tauri ne produit pas de paquet pacman. Le job `arch` reconditionne donc le `.deb` du job
Linux avec [`packaging/arch/PKGBUILD`](../packaging/arch/PKGBUILD) (`makepkg --nodeps`,
sous un utilisateur dédié : `makepkg` refuse de tourner en root). Rien n'est recompilé,
le binaire est celui du `.deb`. Le job vérifie que le paquet porte `LICENSE` et les
licences redistribuées, et ajoute `/usr/share/licenses/candilog/LICENSE`, l'emplacement
attendu par pacman. `publish` attend ce job : une release n'est jamais publiée sans le
paquet Arch.

Chaque paquet embarque `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md` et, sous
`licenses/`, le texte intégral des licences des composants redistribués : OFL 1.1 des
polices IBM Plex et Apache-2.0 de Material Symbols (`bundle.resources`).
La PolyForm impose de transmettre ses termes et sa mention à qui reçoit une copie du
logiciel ; l'OFL et l'Apache-2.0 exigent en plus que **leur** texte accompagne l'œuvre
redistribuée. Un renvoi vers un chemin du dépôt ne suffit pas : la personne qui installe un
paquet n'a pas le dépôt. La vérification est à l'étape 4 de la procédure de release.

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

Sur Arch Linux (et toute distribution hors familles Debian et Red Hat), l'application ne
retient aucun installateur et ouvre la page de la release : un paquet pacman ne s'ouvre
pas d'un double-clic, il s'installe par `sudo pacman -U`.

Le frontend ne désigne ni l'URL ni le nom du fichier : `settings_download_update` ne prend
aucun argument et re-résout l'asset côté Rust. Le paquet est retenu en mémoire (plafonné à
256 Mio), son empreinte SHA-256 est comparée à celle publiée dans `SHA256SUMS`, et il n'est
écrit sur disque qu'ensuite — sous un nom libre, jamais en écrasant un homonyme déjà présent
dans le dossier Téléchargements.

La mise à jour est **assistée, pas automatique** : le téléchargement, l'installation et le
redémarrage restent entre les mains de l'utilisateur. Aucune mise à jour silencieuse n'est
exécutée.

La **disponibilité**, elle, est vérifiée seule. Au démarrage, la coque appelle
`settings_check_update_if_due` ; `SettingsService::check_update_if_due` n'interroge l'API
GitHub qu'une fois par jour au plus (`DELAI_VERIFICATION_MAJ`), l'instant de chaque tentative
étant retenu dans `app_kv` sous `last_update_check`. L'horodatage est écrit **avant** l'appel
et quelle qu'en soit l'issue : une panne réseau ou un quota GitHub dépassé ne doit pas
provoquer une tentative à chaque lancement. Un horodatage illisible ou situé dans le futur
(horloge déréglée) vaut « à vérifier », faute de quoi l'application resterait bloquée dessus.

Une version trouvée s'annonce par un toast — « Candilog 1.4.0 est disponible · Réglages ›
Mises à jour ». Sans bouton : le design l'interdit (`docs/DESIGN.md` §7), le toast nomme donc
sa destination. Une vérification de démarrage qui échoue ne remonte rien ; l'écran Mises à
jour reste le seul endroit où l'on vérifie sur demande — et là, l'erreur s'affiche.

Le contrat de nommage entre le workflow et `updater.rs` est verrouillé par un test
(`les_assets_du_workflow_sont_ceux_que_l_application_attend`) : renommer un asset ici casse
la suite de tests, et non la mise à jour du premier utilisateur.

## Chaîne de confiance

Trois garanties distinctes, la signature de code étant en place sur macOS seulement :

| Garantie | État | Ce qu'elle établit |
| --- | --- | --- |
| `SHA256SUMS` | **en place** | Le fichier téléchargé est intact. L'application le vérifie avant d'ouvrir un installateur ; l'utilisateur peut le refaire à la main. |
| Attestation de provenance Sigstore | **en place** | Le binaire a été construit par ce dépôt, depuis ce commit, par `release.yml`. Vérifiable par `gh attestation verify <fichier> --repo alexandrebouttierdev/candilog`. |
| Signature de code macOS | **en place** | Signé *Developer ID Application* et notarié par Apple : Gatekeeper ouvre le binaire sans intervention. |
| Signature de code Windows | **absente** | Seule reconnue par SmartScreen. Demande un certificat commercial. |

L'attestation est produite par `actions/attest-build-provenance` dans le job `publish`, qui
exige les permissions `id-token: write` et `attestations: write`. Elle est **gratuite** et
ne demande aucun secret : le jeton OIDC du workflow suffit. Elle ne fait pas disparaître les
avertissements des systèmes d'exploitation — rien de gratuit ne le fait.

### Signature de code

**macOS** — en place. Ce n'est pas `tauri-action` qui signe : il se contente de lancer
`tauri build`, et c'est le CLI Tauri qui lit les six secrets du dépôt, importe le
certificat dans un trousseau éphémère, signe le binaire universel et le `.dmg`, puis
notarise et agrafe le `.app`. Le job `build` macOS les exporte ; rien n'est à faire
ailleurs dans le workflow :

- `APPLE_CERTIFICATE` — certificat *Developer ID Application* exporté en `.p12` depuis le
  Trousseau, encodé en base64 sur une seule ligne ;
- `APPLE_CERTIFICATE_PASSWORD` — mot de passe du `.p12` ;
- `APPLE_SIGNING_IDENTITY` — identité exacte du certificat, telle que la nomme
  `security find-identity -v -p codesigning` ;
- `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_CONTENT` — la notarisation
  s'authentifie par clé API App Store Connect (JWT) : l'identifiant de la clé, l'Issuer
  ID, et le contenu du fichier `.p8` téléchargé une seule fois sur App Store Connect.
  Le mot de passe d'application n'est pas utilisable en CI : Apple le refuse (401)
  depuis les adresses des runners, qu'il ne reconnaît pas comme la session du compte.

Tant que `APPLE_CERTIFICATE` est absent, le build reste **non signé sans échec** : la
garde du workflow n'exporte rien, et le CLI saute la signature quand la variable manque.
Un certificat expiré ou révoqué fait, lui, échouer le build — c'est voulu, une release
ne doit pas sortir « silencieusement non signée » alors qu'elle aurait dû l'être.

**Windows** — encore absent. Certificat de signature de code OV (~300 €/an) ou EV (jeton
matériel, exigé depuis 2023 pour une réputation SmartScreen immédiate). Le jour où il est
acquis, renseigner `bundle.windows.signCommand` dans `src-tauri/tauri.conf.json`, ou
ajouter au job Windows une étape `signtool` après `tauri-action`, avant la préparation des
assets renommés. Le certificat OV n'annule pas SmartScreen tout de suite : la réputation
se construit au fil des téléchargements.

Le jour où le certificat Windows est acquis, mettre à jour dans la même tâche le tableau
ci-dessus, la note du `README`, et les notes de release du workflow.

## Côté site

Le site (`website/lib/data/plateformes.ts`) pointe directement sur
`…/releases/latest/download/candilog-<plateforme>-latest.<ext>` pour toujours servir la
dernière version publiée.

## Procédure de release

0. Lancer les validations de `docs/CODE_RULES.md` §20 **plus** `yarn tauri build`.
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
   plateforme (Arch Linux compris), et présence de `SHA256SUMS` — sans lui, la mise à jour in-app refusera
   d'ouvrir l'installateur.
4. Installer au moins un paquet sur une machine propre et vérifier que
   `/usr/lib/Candilog/LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md` et
   `/usr/lib/Candilog/licenses/ibm-plex-OFL-1.1.txt` y figurent.
