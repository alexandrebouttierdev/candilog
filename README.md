# Candilog

Application desktop de suivi de recherche d'emploi : candidatures, entreprises, contacts,
entretiens, relances, CV et lettres de motivation, avec assistance IA locale ou distante.

**Tauri 2 + React 19 + TypeScript** pour l'interface, **Rust** pour le métier, **SQLite**
local pour les données. Tout reste sur la machine de l'utilisateur : aucune donnée n'est
envoyée ailleurs, hors appel explicite à un fournisseur IA distant configuré par
l'utilisateur.

Version 0.0.1. Le dépôt contient également [`website/`](website/), le site public
candilog.fr, projet Next.js autonome.

## Fonctionnalités

- Suivi des candidatures : kanban ou table, filtres et recherche côté base, historique de
  statut, export CSV.
- Répertoire d'entreprises et de contacts, avec héritage des valeurs de l'entreprise
  (ville, adresse, type) sur la candidature.
- Calendrier des entretiens et des relances.
- Profil professionnel, génération de CV et de lettres de motivation en PDF A4 une page,
  analyse ATS déterministe.
- Fournisseurs IA au choix : Ollama (local), Claude, OpenAI, Gemini, Mistral, Nvidia ou
  point de terminaison personnalisé. La clé API vit dans le coffre du système.
- Sauvegarde et restauration de la base, mise à jour assistée depuis les GitHub Releases.

## Installer

Les binaires sont publiés dans les [releases GitHub](https://github.com/alexandrebouttierdev/candilog/releases/latest)
de ce dépôt.

| Système | Fichier | Installation |
| --- | --- | --- |
| Windows 10 / 11 | `candilog-windows-latest.exe` | Double-cliquer sur l'installateur |
| macOS (Apple Silicon et Intel) | `candilog-macos-latest.dmg` | Ouvrir l'image, glisser Candilog dans *Applications* |
| Ubuntu, Debian | `candilog-ubuntu-latest.deb` | `sudo apt install ./candilog-ubuntu-latest.deb` |
| Fedora, RHEL | `candilog-fedora-latest.rpm` | `sudo dnf install ./candilog-fedora-latest.rpm` |

### Vérifier le fichier téléchargé

Chaque release publie `SHA256SUMS`. Téléchargez-le à côté de votre installateur, puis :

```bash
sha256sum -c SHA256SUMS --ignore-missing     # Linux
shasum -a 256 -c SHA256SUMS --ignore-missing # macOS
```

```powershell
Get-FileHash .\candilog-windows-latest.exe -Algorithm SHA256   # Windows, à comparer à la ligne du fichier
```

L'empreinte atteste que le fichier est arrivé **intact**. Pour vérifier d'où il vient,
chaque binaire porte une **attestation de provenance** [Sigstore](https://www.sigstore.dev/),
produite par le workflow de release et vérifiable avec la [CLI GitHub](https://cli.github.com/) :

```bash
gh attestation verify candilog-ubuntu-latest.deb --repo alexandrebouttierdev/candilog
```

La commande confirme cryptographiquement que ce fichier précis a été construit par ce
dépôt, depuis un commit donné, par le workflow `release.yml` — et non recompilé ou modifié
par un tiers. C'est la garantie d'origine la plus forte que le projet puisse offrir
aujourd'hui.

### Avertissement « éditeur inconnu »

Les deux vérifications ci-dessus ne suppriment **pas** les avertissements de Windows et de
macOS : ces systèmes ne reconnaissent que la signature de code, qui passe par un certificat
commercial que le projet n'a pas encore acquis.

- **Windows** : SmartScreen affiche « Windows a protégé votre ordinateur ». Le bouton
  d'exécution est derrière *Informations complémentaires* → *Exécuter quand même*.
- **macOS** : Gatekeeper refuse l'ouverture. Faire un **clic droit** sur l'application →
  *Ouvrir*, puis confirmer ; ou autoriser depuis *Réglages Système → Confidentialité et
  sécurité*.
- **Linux** : aucun avertissement particulier.

| Garantie | Ce qu'elle prouve | Ce qu'elle ne prouve pas |
| --- | --- | --- |
| `SHA256SUMS` | Le fichier est arrivé intact | Son origine |
| Attestation de provenance | Il a été construit par ce dépôt, ce commit, ce workflow | Rien pour SmartScreen ni Gatekeeper |
| Signature de code | *(absente)* | — |

Si vous préférez ne dépendre d'aucune de ces confiances, le code est ici : compilez-le
vous-même en suivant la section suivante.

### Mises à jour

Candilog ne vérifie **jamais** les mises à jour tout seul. *Réglages → Mises à jour →
Rechercher maintenant* interroge l'API GitHub, télécharge l'installateur adapté à votre
système, **vérifie son empreinte SHA-256** contre `SHA256SUMS` et ne l'ouvre qu'ensuite.
L'installation reste votre geste : aucune mise à jour silencieuse.

## Vos données

Tout vit sur votre machine. Rien n'est envoyé nulle part, sauf deux cas que vous déclenchez
vous-même : la recherche de mise à jour ci-dessus, et les appels au fournisseur IA que vous
avez configuré. Il n'y a ni télémétrie, ni statistiques d'usage, ni rapport d'erreur
automatique.

| Quoi | Où |
| --- | --- |
| Base, journaux, exports | Linux `~/.local/share/fr.candilog.desktop/` · Windows `%APPDATA%\fr.candilog.desktop\` · macOS `~/Library/Application Support/fr.candilog.desktop/` |
| Clé API du fournisseur IA | Trousseau du système (`fr.candilog.desktop`), jamais dans la base ni dans les journaux |
| CV, lettres, sauvegardes, CSV exportés | À l'emplacement que vous choisissez dans la fenêtre d'enregistrement |

Ce qui part chez un fournisseur IA **distant** quand vous lancez une génération : le texte
de l'offre, et les éléments de votre profil nécessaires à l'opération (identité,
expériences, formations, compétences) ou le texte du CV que vous importez. Avec **Ollama**,
le fournisseur par défaut, rien ne quitte la machine.

*Réglages → Sauvegardes* exporte l'intégralité de la base en un fichier, et efface vos
données à la demande. La désinstallation du paquet ne supprime ni le dossier de données ni
l'entrée de trousseau ci-dessus : les retirer à la main les efface définitivement.

## Développer

Prérequis : Node.js LTS, Rust 1.91, et les dépendances système de Tauri 2. Détails dans
[`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md).

```bash
npm install
npm run tauri dev
```

Le frontend seul (sans fenêtre native, IPC indisponible) :

```bash
npm run dev
```

## Validations

Aucune CI ne vérifie la qualité — le seul workflow publie les releases. Ces commandes sont
à lancer localement.

```bash
npm run lint
npm test
npm run build            # inclut tsc --noEmit

cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
cargo deny --manifest-path src-tauri/Cargo.toml check
```

## Architecture

```
React                          Tauri IPC                      Rust
─────                          ─────────                      ────
View                                                          Command
 ↓                                                             ↓
ViewModel (hook)                                              Application Service
 ↓                                                             ↓
Frontend Service ──────────────► invoke ──────────────────►   Domain
                                                               ↓
                                                              Repository (trait)
                                                               ↓
                                                              Infrastructure (SQLite, HTTP, IA)
```

### Frontend — `src/`

Feature-first + MVVM. Chaque feature est autonome :

```
features/<feature>/
├── model/       types, schémas Zod, constantes métier
├── view/        pages et composants React
├── viewmodel/   hooks d'orchestration UI
└── services/    seule couche connaissant les commandes Tauri de la feature
```

`shared/ui/` ne contient que du générique ; un composant métier reste dans sa feature.

Les appels IPC passent tous par `shared/services/ipc.ts` — une règle ESLint interdit
d'importer `invoke` ailleurs.

### Backend — `src-tauri/src/`

Architecture hexagonale pragmatique, feature-first :

```
app/              état partagé (AppState) et démarrage
core/             config, base de données, erreurs, journal, pagination,
                  sauvegardes, mises à jour, coffre à secrets
features/<f>/     domain · application · infrastructure · presentation
infrastructure/   export PDF (CV et lettres)
migrations/       schéma SQLite embarqué (init_schema.sql)
```

Le `domain` ne dépend ni de Tauri, ni de rusqlite, ni d'un fournisseur IA.

### Contrat IPC

Les types TypeScript des DTO sont **générés** depuis Rust par `ts-rs` dans
`src/shared/types/generated/`. Ils ne s'éditent pas à la main : régénérer avec

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Un DTO Rust modifié sans régénération fait échouer `npm run build`.

### Erreurs

Le backend rejette toujours `{ code, message }` : `code` est stable et destiné au
branchement conditionnel, `message` est rédigé pour l'utilisateur. Le détail technique
part au journal, jamais à l'écran.

## Base de données

SQLite, `rusqlite` + `r2d2`, schéma embarqué dans le binaire et appliqué par
`PRAGMA user_version`. Une base d'une génération antérieure est refusée en lecture seule
plutôt que migrée automatiquement.

`CANDILOG_DATA_DIR` déplace le dossier de données. En développement, la base vit sous
`src-tauri/.candilog-dev/` (ancrée sur le manifeste Cargo) et ne touche jamais aux
données réelles. `RUST_LOG` règle le niveau de journalisation (`candilog=info` par défaut).

## Documentation

| Document | Sujet |
| --- | --- |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Couches et frontières |
| [`docs/CODE_RULES.md`](docs/CODE_RULES.md) | Qualité, conventions, tests, sécurité |
| [`docs/DESIGN.md`](docs/DESIGN.md) | Design system de l'application |
| [`docs/DATA.md`](docs/DATA.md) | Schéma SQLite, référentiels, chemins de données |
| [`docs/AI.md`](docs/AI.md) | Fournisseurs IA, streaming, annulation |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | Installation, exécution, validations |
| [`docs/RELEASES.md`](docs/RELEASES.md) | Publication des binaires |
| [`CHANGELOG.md`](CHANGELOG.md) | Journal des versions publiées |
| [`SECURITY.md`](SECURITY.md) | Signaler une vulnérabilité |
| [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) | Attribution des composants tiers redistribués |

Les binaires sont publiés en GitHub Release sur ce dépôt à chaque push sur `master`.

### Agents IA

[`AGENTS.md`](AGENTS.md) est la référence commune (Codex, OpenCode, Cursor et tout agent
lisant ce format) ; [`CLAUDE.md`](CLAUDE.md) est le point d'entrée Claude Code et
l'importe. Des fichiers `AGENTS.md` imbriqués précisent les règles propres à
[`src/`](src/AGENTS.md), [`src-tauri/`](src-tauri/AGENTS.md) et
[`website/`](website/AGENTS.md).

## Licence

Candilog est un projet **source available avec double licence**.

### Usage non commercial

Le code Candilog est mis à disposition sous la **PolyForm Noncommercial License 1.0.0** pour les usages autorisés par cette licence. Consultez le [texte officiel](./LICENSE), qui reste la référence.

### Usage commercial

Toute utilisation commerciale nécessite une licence commerciale séparée, accordée explicitement par le titulaire des droits. Consultez [`COMMERCIAL_LICENSE.md`](./COMMERCIAL_LICENSE.md) pour comprendre la démarche ; ce document n'accorde pas à lui seul de droits commerciaux.

Les licences des dépendances tierces sont traitées dans [`LICENSES.md`](./LICENSES.md), et
les composants redistribués avec les binaires sont attribués dans
[`THIRD_PARTY_NOTICES.md`](./THIRD_PARTY_NOTICES.md).

### Sécurité

Pour signaler une vulnérabilité, suivre [`SECURITY.md`](./SECURITY.md) — pas d'issue
publique.

### Contributions

Les contributions sont les bienvenues lorsqu'elles respectent les règles décrites dans [`CONTRIBUTING.md`](./CONTRIBUTING.md) et le mécanisme de contribution et de licence prévu par le projet, notamment le [`CLA.md`](./CLA.md) lorsqu'il est requis.

Copyright © 2026 Alexandre Bouttier
