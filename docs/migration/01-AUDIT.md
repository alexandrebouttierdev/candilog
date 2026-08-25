# Audit de migration — Candilog Iced → Tauri 2 + React

> Étape 51 de `MIGRATION.md`. Aucun code n'est écrit avant validation de ce document.
> Date : 2026-08-25 · Base : `dev` @ `1597da6` · 45 373 lignes Rust.

---

## 0. Verdict en une page

Le projet est **beaucoup plus prêt que ne le laisse craindre le mot « migration »**.

| Mesure | Lignes | Part |
|---|---:|---:|
| Rust hors tests **sans aucune référence à Iced** | 15 563 | 42 % |
| Rust hors tests **couplé à Iced** | 21 292 | 58 % |
| Tests | 8 518 | — |

La séparation métier / UI a déjà été faite dans le projet Iced : **tous** les fichiers
`model.rs`, `repository.rs`, `service.rs` des 12 modules métier sont Iced-free, sérialisables
Serde, et les services sont déjà génériques sur un trait de dépôt.

```rust
pub struct CandidatureService<R: CandidatureRepository> { repo: R }
```

`src/shared/state.rs::AppState` est déjà, mot pour mot, l'`AppState` décrit au §23 du prompt :
un service par feature, le pool SQLite, le coffre à secrets, la table des générations IA
annulables. Il est réutilisable **tel quel** comme état managé Tauri.

`src/shared/error.rs::AppError` (thiserror, 7 variantes, `message_utilisateur()` séparé du
`Display` technique) couvre déjà le §27 ; il lui manque uniquement un `code` stable pour le
contrat IPC.

**Conséquence sur le plan de charge :** la migration n'est pas une réécriture du backend.
C'est (a) une **suppression** de la couche Iced, (b) l'**ajout** d'une couche
`presentation/commands.rs` par feature, (c) une **réécriture complète du frontend**, qui
concentre l'essentiel de l'effort réel.

---

## 1. Cartographie des modules existants

### 1.1 Modules métier (`src/modules/`)

Chaque module suit déjà `model.rs` / `repository.rs` / `service.rs` + `views/` + `components/`.
Les trois premiers migrent, les deux derniers disparaissent.

| Module | Métier réutilisable (lignes) | Iced à jeter (lignes) | Accès données | Destination |
|---|---:|---:|---|---|
| `candidatures` | 898 | 1 397 | SQLite (`candidatures`, `statut_history`) | `features/candidatures/{domain,application,infrastructure}` |
| `entreprises` | 450 | 488 | SQLite (`entreprises`) | `features/entreprises/…` |
| `contacts` | 405 | 483 | SQLite (`contacts`) | `features/contacts/…` |
| `entretiens` | 448 | 860 | SQLite (`entretiens`) | `features/entretiens/…` |
| `relances` | 296 | 119 | SQLite (`relances`) | `features/relances/…` |
| `secteurs` | 223 | 0 | SQLite (`secteurs_activite`) | `features/secteurs/…` |
| `cv` | 236 | 465 | SQLite (`cv_versions`) | `features/cv/…` |
| `lettres` | 190 | 184 | SQLite (`lettres_motivation`) | `features/lettres/…` |
| `profil` | 198 + `shared/profile.rs` (171) | 1 506 | SQLite (`profil`, JSON) | `features/profil/…` |
| `settings` | 235 | 786 | SQLite (`parametres`, JSON) + keyring | `features/parametres/…` |
| `metriques` | 505 | 1 452 | SQLite (`llm_appels`, `scores_ats`) | `features/metriques/…` |
| `ia` | 3 415 | 3 220 | SQLite (`cache_ia`) + HTTP | `features/ia/` + `infrastructure/ai/` |

Détail du module `ia`, le plus gros bloc métier et **entièrement Iced-free** :

| Fichier | Lignes | Rôle | Destination |
|---|---:|---|---|
| `cv_engine.rs` | 629 | Moteur de génération de CV par étapes | `features/ia/application/` |
| `profile_extraction.rs` | 669 | Extraction de profil depuis un PDF | `features/ia/application/` |
| `service.rs` | 507 | 6 cas d'usage IA (offre, CV, lettre, entretien, import) | `features/ia/application/service.rs` |
| `cv_model.rs` / `cv_sections.rs` / `cv_document.rs` | 845 | Modèle de CV structuré | `features/ia/domain/` |
| `providers/` (ollama, claude, gemini, openai_compat) | 959 | 4 implémentations `LlmProvider` | `infrastructure/ai/providers/` |
| `provider.rs` / `factory.rs` / `mode.rs` | 398 | Trait `LlmProvider`, `GenOptions`, `AnalysisMode` | `features/ia/domain/` + `infrastructure/ai/` |
| `scoring.rs` / `grounding.rs` / `contacts.rs` / `cache.rs` | 549 | Score ATS, anti-hallucination, cache SQLite | `features/ia/{application,infrastructure}` |

L'abstraction `AiProvider` demandée au §29 **existe déjà** (`trait LlmProvider` + 4 impls +
`ProviderKind` à 7 variantes). Rien à concevoir, seulement à déplacer.

### 1.2 Couche transverse (`src/shared/`, `src/core/`)

| Fichier | Iced ? | Destination |
|---|---|---|
| `shared/state.rs` (`AppState`) | non | `app/state.rs` — repris tel quel, devient `tauri::State` |
| `shared/error.rs` (`AppError`) | non | `core/errors/app_error.rs` — **+ ajout d'un `code` IPC** |
| `shared/db.rs` (pool r2d2 + 8 migrations embarquées) | non | `core/database/` — repris tel quel |
| `shared/sqlite.rs` (helpers de dépôt) | non | `core/database/helpers.rs` |
| `shared/llm.rs` (`LlmConfig`, `ProviderKind`, `AnalysisMode`) | non | `features/parametres/domain/` |
| `shared/profile.rs` (`PersonalInfo`, `Experience`, `Skill`, `Education`…) | non | `features/profil/domain/` |
| `shared/secrets.rs` (keyring) | non | `infrastructure/secure_storage/` |
| `shared/http.rs`, `shared/pdf.rs`, `shared/validation.rs`, `shared/types.rs` | non | `infrastructure/http`, `infrastructure/pdf`, `core/utils` |
| `core/cv_pdf.rs` (784 l., export PDF printpdf) | non | `infrastructure/pdf/cv_pdf.rs` |
| `core/backup.rs` (sauvegarde/validation SQLite) | non | `features/parametres/application/backup.rs` |
| `core/updater.rs` (MàJ GitHub) | non | `features/parametres/application/updater.rs` |
| `core/config.rs` (chemins de données) | non | `core/config/app_config.rs` |
| `core/external.rs` (ouverture de liens) | non | remplacé par le plugin `tauri-plugin-opener` |
| `core/logging.rs` | non | `core/logging.rs` (tracing conservé) |
| `core/theme_systeme.rs` (détection thème OS, ashpd/winreg) | non | **supprimé** — `prefers-color-scheme` côté CSS |

### 1.3 Couche à supprimer intégralement

| Répertoire | Lignes | Motif |
|---|---:|---|
| `src/ui/` (theme, components, format) | 4 060 | Design system Iced — remplacé par Tailwind + SPECDESIGN |
| `src/app/` (state, message, update, view) | 4 600 | Boucle Elm d'Iced — remplacée par React + TanStack Query |
| `src/navigation/` | 362 | Routeur typé Iced — remplacé par React Router (**mais voir §3.1 : c'est la carte des écrans, à conserver comme référence**) |
| `src/modules/*/views/` + `*/components/` | ~9 000 | Widgets Iced |

**Trois exceptions à récupérer avant de jeter `src/app/` :**

| Fichier | Lignes | Ce qu'il contient de non-Iced |
|---|---:|---|
| `app/export.rs` | 49 | Export CSV des candidatures filtrées → `features/candidatures/application/export.rs` |
| `app/snapshot.rs` | 315 | Construction paginée des instantanés (agrégations dashboard) → à répartir dans les services |
| `app/profile_edit.rs` | 430 | Mutations structurées du brouillon de profil + décisions ATS → règles métier à extraire vers `features/profil/application/` |

`app/update/operations/` (candidates.rs, documents.rs) et `app/update/forms.rs` mélangent
orchestration Iced et enchaînements métier : à lire feature par feature pour ne rien perdre,
mais **rien n'y est à porter tel quel**.

---

## 2. Base de données

8 migrations, schéma stable, **à conserver intégralement** (§24). 12 tables :

```
entreprises ──┬─< contacts ──┐
              │              │
              └─< candidatures ──┬─< statut_history
                    │            ├─< relances
                    │            └─< entretiens
                    └── secteurs_activite (FK entreprises.secteur_id)

cv_versions · lettres_motivation · profil(1) · parametres(1)
llm_appels · scores_ats · cache_ia · app_kv
```

Contraintes `CHECK` en base à refléter dans les schémas Zod **et** dans le domaine Rust :

- `candidatures.statut` ∈ `EN_ATTENTE | RELANCEE | ENTRETIEN | REFUS`
- `candidatures.type_contrat` ∈ `CDI | CDD | Freelance | Stage | Alternance | Interim | Autre`
- `entretiens.type` ∈ `Présentiel | Visio | Téléphonique | Technique | RH | Autre`

### Point de décision : rusqlite vs SQLx

Le §24 dit « utiliser **de préférence** SQLx ». L'existant utilise `rusqlite` + `r2d2`, en
**synchrone**, sur ~2 700 lignes de dépôts SQL écrits à la main, couverts par des tests.

| | rusqlite + r2d2 (existant) | SQLx |
|---|---|---|
| Réécriture | 0 ligne | ~2 700 lignes de dépôts + tests |
| Risque de régression | nul | élevé (toutes les requêtes retouchées) |
| Async | via `tauri::async_runtime::spawn_blocking` | natif |
| Migrations | déjà embarquées, `user_version`, `foreign_key_check` | à réécrire |
| Vérification SQL à la compilation | non | oui (`query!`) |

**Recommandation : conserver rusqlite + r2d2**, et rendre les commandes Tauri asynchrones en
enveloppant les appels de service dans `spawn_blocking` (§28 : ne jamais bloquer le thread
principal). Le bénéfice de SQLx ne compense pas la réécriture d'une couche déjà testée et
fonctionnelle. → **à arbitrer.**

---

## 3. Analyse de SPECDESIGN

### 3.0 Avertissement

`SPECDESIGN/` a été rédigé pour une cible **.NET / Avalonia** (cf. `PROMPT-claude-code.md` et
`Guide Avalonia.dc.html`). Les **tokens, maquettes, états et règles UX sont agnostiques** et
restent la source de vérité (§34). Seules les colonnes « Base Avalonia » et les extraits XAML
sont à retraduire : `ControlTemplate` → composant React, `DynamicResource` → variable CSS,
`INotifyDataErrorInfo` → Zod + React Hook Form, `IDialogService` → composant `ConfirmDialog`.

Le `.claude/skills/avalonia*` présent dans le dépôt vise la même cible abandonnée : sans objet
pour cette migration.

### 3.1 Navigation — correspondance exacte avec l'existant

Le rail des maquettes est **identique** à `src/navigation/mod.rs`. Sept sections, seize routes :

| Section (rail) | Onglets contextuels | Route Rust existante | Route React |
|---|---|---|---|
| Accueil | — | `Dashboard` | `/` |
| Suivi | Candidatures (Kanban/Liste), Calendrier | `Candidatures`, `Calendrier` | `/suivi/candidatures`, `/suivi/calendrier` |
| Relations | Entreprises, Réseau | `Entreprises`, `Reseau` | `/relations/entreprises`, `/relations/reseau` |
| Documents | Mes CV, Générer un CV, Mes lettres, Lettre de motivation, Analyser | `Cv`, `CvGenerator`, `Lettres`, `LettreMotivation`, `CvImport` | `/documents/*` |
| Analyses | — (2 onglets internes) | `Statistiques` | `/analyses` |
| Profil | 4 onglets internes | `Profil` | `/profil` |
| Réglages | IA, Sauvegardes, Mises à jour, À propos | `Parametres`, `Sauvegardes`, `MisesAJour`, `APropos` | `/reglages/*` |

`src/navigation/mod.rs` est donc à **lire comme spécification** avant d'être supprimé.

### 3.2 Design tokens → configuration Tailwind

14 tokens de couleur, clair et sombre, à traduire en variables CSS + thème Tailwind :

| Token | Rôle | Clair | Sombre |
|---|---|---|---|
| `Brush.Page` | Fond d'application | `#f6f7fa` | `#15181f` |
| `Brush.Surface` | Cartes, tableaux, modales | `#ffffff` | `#1d2028` |
| `Brush.SurfaceAlt` | Rail, en-têtes, pieds | `#fafbfd` | `#191c23` |
| `Brush.Border` | Filet standard | `#e2e6ee` | `#343943` |
| `Brush.BorderStrong` | Filet accentué | `#c9cfdb` | `#474d59` |
| `Brush.Text.Primary` | Texte principal | `#22262f` | `#f1f3f7` |
| `Brush.Text.Secondary` | Libellés, valeurs | `#666d7d` | `#a5abb8` |
| `Brush.Text.Tertiary` | Métadonnées, aides | `#8b91a0` | `#7c8290` |
| `Brush.Accent` | Action primaire, sélection | `#2957d8` | `#7b9bff` |
| `Brush.Accent.Tint` | Fond d'état sélectionné | `#eaefff` | `#25304d` |
| `Brush.Success` | Entretien, réussite | `#1c7a4f` | `#5fd598` |
| `Brush.Warning` | Relance, échéance | `#9a6a12` | `#e5b65a` |
| `Brush.Danger` | Refus, suppression | `#c0392f` | `#f08b7f` |
| `Brush.Neutral.Tint` | Pastilles, pistes de jauge | `#f2f4f8` | `#262a33` |

Nommage Tailwind cible : `bg-surface`, `text-secondary`, `border-strong`, `bg-accent-tint`,
`text-success`… Une seule couleur d'accent, aucun littéral hexadécimal dans les composants (§8).

Typographie (7 niveaux), espacement (échelle `4 · 8 · 12 · 16 · 20 · 28`), rayons
(carte 12 px, champ 9 px, bouton 8 px, pastille 6 px), hauteurs de contrôle (bouton 33 px,
champ 36 px, ligne de tableau 44 px), 4 niveaux d'élévation : tous à porter en tokens.

### 3.3 Bibliothèque de composants (16, définis par le guide)

| Composant | Destination React |
|---|---|
| `AppShell`, `NavRailItem`, `ContextTab`, `PageHeader` | `app/layout/` |
| `PrimaryButton` / `GhostButton`, `FormField`, `EmptyState`, `ErrorBanner`, `StatusPill`, `Pager`, `DataTable`, `ModalHost`, `DetailDrawer`, `TimelineList` | `shared/ui/` |
| `StatCard` | `shared/ui/` (générique) |
| `KanbanBoard`, `EntityPicker` | métier → `features/candidatures/view/components/`, `shared/ui/EntityPicker` (générique, paginé) |

### 3.4 Règles transverses à respecter (§7 du guide)

Focus clavier accent toujours visible · Échap ferme / Ctrl+Entrée valide · cibles ≥ 32 px
(44 px principales) · contraste 4,5:1 · une seule zone scrollable par colonne · virtualisation
> 50 éléments · responsive 1024→2560 px (rail réduit < 1200 px, colonnes latérales repliées
< 1100 px) · transitions 120–180 ms sur fond/bordure/opacité uniquement · toasts 4 s en bas à
droite · actions destructives rouges, isolées, toujours confirmées.

Cinq états par écran, obligatoires : **loading, empty, error, success, populated**.

---

## 4. Inventaire des formulaires

Sources croisées : `Modales.dc.html` (maquettes), `src/app/state/forms.rs` (état Iced actuel),
schéma SQL, validations `service.rs`.

### 4.1 Formulaires principaux (modales)

| # | Formulaire | Feature | Champs (maquette) | Règles de validation | Schéma Zod | DTO Rust | Command Tauri |
|---|---|---|---|---|---|---|---|
| 1 | Nouvelle / éditer candidature | `candidatures` | poste\*, entreprise\* (picker paginé), contrat, statut, date d'envoi\*, lien de l'offre, notes | poste non vide ; date `JJ-MM-AAAA` valide ; lien = URL http(s) ; entreprise existante | `candidature-form.schema.ts` | `NouvelleCandidature` | `candidature_creer`, `candidature_modifier` |
| 2 | Nouvelle / éditer entreprise | `entreprises` | nom\*, secteur (référentiel), type, site web, ville, adresse, notes | nom non vide (seule obligation) ; site web = URL http(s) | `entreprise-form.schema.ts` | `NouvelleEntreprise` / `MajEntreprise` | `entreprise_creer`, `entreprise_modifier` |
| 3 | Nouveau / éditer contact | `contacts` | prénom\*, nom\*, e-mail, téléphone, entreprise, poste, **rôle dans le suivi**, LinkedIn | prénom + nom non vides ; e-mail valide si fourni | `contact-form.schema.ts` | `NouveauContact` / `MajContact` | `contact_creer`, `contact_modifier` |
| 4 | Nouvel / éditer entretien | `entretiens` | candidature\*, interlocuteur, **date\* + heure séparées**, format, **lien / lieu**, préparation | candidature existante ; date+heure valides ; format ∈ enum | `entretien-form.schema.ts` | `NouvelEntretien` | `entretien_creer`, `entretien_modifier` |
| 5 | Nouvelle relance | `relances` | candidature\*, date\*, canal, message | candidature existante ; date valide | `relance-form.schema.ts` | `NouvelleRelance` | `relance_creer`, `relance_modifier` |
| 6 | Confirmation de suppression | transverse | — | — | — | — | `*_supprimer` |

**Écarts maquette ↔ base à arbitrer :**

- `contact.rôle dans le suivi` : **absent de la table `contacts`**. Soit colonne ajoutée
  (migration 009), soit champ retiré de la maquette. → décision requise.
- `entretien` : la maquette sépare **Date** et **Heure**, et fusionne **« lien / lieu »**.
  La base a `date_entretien TEXT` et `lieu TEXT` : composition/décomposition dans le mapper,
  pas de migration nécessaire.
- `entretien.compte_rendu` et `analyse_ia` : présents en base et dans l'app Iced, **absents de
  la maquette de création** → à placer dans le panneau de détail, pas dans la modale de création.
- `candidature.contact_id` : en base, absent de la modale → à exposer dans le détail.

### 4.2 Formulaires secondaires (non modaux)

| # | Formulaire | Feature | Règles | Schéma Zod |
|---|---|---|---|---|
| 7 | Filtres candidatures | `candidatures` | statut, contrat, entreprise, ville, poste, période (7 critères, cumulables) | `candidature-filter.schema.ts` |
| 8 | Réglages IA | `parametres` | provider ∈ 7, clé API (coffre), endpoint (URL, requis si Custom), modèle, température 0–2, mode d'analyse | `llm-config.schema.ts` |
| 9 | Préférences | `parametres` | thème `light\|dark\|system`, langue | `preferences.schema.ts` |
| 10 | Profil — identité | `profil` | prénom, nom, e-mail\*, téléphone, ville, accroche, résumé, LinkedIn, GitHub, site | `profil-identite.schema.ts` |
| 11 | Profil — expérience | `profil` | intitulé\*, entreprise\*, lieu, début\*, fin, poste actuel, description | `experience.schema.ts` |
| 12 | Profil — formation | `profil` | diplôme\*, établissement\*, lieu, dates, description | `formation.schema.ts` |
| 13 | Profil — compétence / langue | `profil` | nom\* | `competence.schema.ts` |
| 14 | Générateur de CV IA | `ia` | offre (texte\*), version source, options | `cv-generation.schema.ts` |
| 15 | Rédaction de lettre IA | `ia` | entreprise, poste, ton, longueur, contexte | `lettre-generation.schema.ts` |
| 16 | Analyse de CV (dépôt PDF) | `ia` | fichier PDF\* (5 états de dépôt) | `cv-import.schema.ts` |
| 17 | Enregistrer une version de CV | `cv` | nom\* | `cv-version.schema.ts` |
| 18 | Sauvegarde / restauration | `parametres` | chemin, confirmation destructive | `backup.schema.ts` |

**Validation conditionnelle** (§13) à porter en Zod, en plus des règles serveur :
`endpoint` requis si `provider = custom` ; `end_date` interdite si `current = true` ;
`lien_offre` requis si le suivi l'exige. La règle `poste` requis + date valide + URL http(s)
existe déjà dans `CandidatureService::valider` et **reste côté Rust** (§14).

---

## 5. Mapping complet ancien → nouveau

```
src/modules/<m>/model.rs        →  src-tauri/src/features/<m>/domain/
src/modules/<m>/repository.rs   →  src-tauri/src/features/<m>/{domain/repository.rs (trait),
                                      infrastructure/sqlite_repository.rs (impl + SQL)}
src/modules/<m>/service.rs      →  src-tauri/src/features/<m>/application/service.rs
                          (nouveau)  src-tauri/src/features/<m>/application/{dto,mapper}
                          (nouveau)  src-tauri/src/features/<m>/presentation/commands.rs
src/modules/<m>/views|components →  ✗ supprimé, remplacé par src/features/<m>/view/

src/shared/state.rs             →  src-tauri/src/app/state.rs        (repris)
src/shared/error.rs             →  src-tauri/src/core/errors/        (+ code IPC)
src/shared/db.rs + migrations/  →  src-tauri/src/core/database/      (repris)
src/shared/{sqlite,http,pdf,validation,types}.rs → core/utils + infrastructure/
src/shared/{llm,profile}.rs     →  features/{parametres,profil}/domain/
src/shared/secrets.rs           →  infrastructure/secure_storage/
src/core/cv_pdf.rs              →  infrastructure/pdf/
src/core/{backup,updater}.rs    →  features/parametres/application/
src/core/config.rs              →  core/config/
src/core/theme_systeme.rs       →  ✗ (CSS prefers-color-scheme)
src/core/external.rs            →  ✗ (tauri-plugin-opener)
src/modules/ia/providers/       →  infrastructure/ai/providers/
src/navigation/mod.rs           →  spécification du routeur React, puis ✗
src/{ui,app}/                   →  ✗

SPECDESIGN/Guide Avalonia       →  tailwind.config + src/shared/ui/
SPECDESIGN/*.dc.html            →  src/features/*/view/pages/
```

### Correspondance feature par feature (§47)

| Rust `features/` | React `features/` | Écran(s) SPECDESIGN |
|---|---|---|
| `candidatures` | `candidatures` | Suivi (Kanban + Liste), Modales #1 |
| `entretiens` | `entretiens` | Suivi (Calendrier), Modales #4 |
| `relances` | `relances` | Calendrier, Analyses, Modales #5 |
| `entreprises` + `secteurs` | `entreprises` | Relations → Entreprises, Modales #2 |
| `contacts` | `contacts` | Relations → Réseau, Modales #3 |
| `cv` + `lettres` | `documents` | Documents → Mes CV / Mes lettres |
| `ia` | `ia` | Documents → Générer / Lettre / Analyser |
| `profil` | `profil` | Profil (4 onglets) |
| `metriques` | `analyses` | Dashboard, Analyses |
| `parametres` | `parametres` | Réglages (4 onglets) |

---

## 6. Décisions arbitrées

| # | Sujet | Décision | Conséquence |
|---|---|---|---|
| A | Emplacement du nouveau projet | **`candilog-tauri/` dans ce dépôt** | L'app Iced reste intacte sous `src/` comme référence (§43) ; comparaison feature par feature possible jusqu'à la fin |
| B | Couche SQLite | **rusqlite + r2d2 conservés** | Zéro réécriture des dépôts et de leurs tests ; les commandes Tauri sont `async` et enveloppent les appels de service dans `tauri::async_runtime::spawn_blocking` (§28) |
| C | Types TypeScript | **`ts-rs`** | `#[derive(TS)]` sur les DTO, `.ts` générés par `cargo test`, aucune dépendance à l'exécution |
| D | `contact.rôle dans le suivi` | **Migration 009** | Colonne `contacts.role_suivi` ajoutée ; le champ de la maquette existe réellement |
| E | `core/theme_systeme.rs` | **Supprimé** | `prefers-color-scheme` couvre le besoin ; `ashpd` et `winreg` disparaissent des dépendances |
| F | Ordre de migration | **Hybride feature / lot design** | Cf. §7 |

## 7. Plan de migration proposé

Chaque tranche est livrable et laisse l'application compilable et testable.

| Tranche | Contenu | Dépendances |
|---|---|---|
| **T0 — Socle** | Scaffold Tauri 2 + Vite/React/TS ; `core/{config,database,errors}` ; `AppState` repris ; migrations SQLite branchées ; `AppError` + code IPC ; capabilities minimales ; `ts-rs` | — |
| **T1 — Design system** | Tokens Tailwind (clair/sombre), `shared/ui/` (16 composants du guide), `AppShell` + rail + onglets + `PageHeader`, React Router, providers TanStack Query / Zustand | T0 |
| **T2 — Entreprises + Secteurs + Contacts** | Feature la plus simple, valide la chaîne complète View→VM→Service→IPC→Rust ; écran Relations ; modales #2 et #3 | T1 |
| **T3 — Candidatures** | Kanban + Liste + filtres + détail ; modale #1 ; export CSV | T2 (dépend d'`entreprises`) |
| **T4 — Entretiens + Relances** | Calendrier ; modales #4 et #5 | T3 |
| **T5 — Dashboard + Analyses** | `metriques`, agrégations reprises de `app/snapshot.rs` | T3, T4 |
| **T6 — Profil** | 4 onglets, complétion, modales #10–13 | T1 |
| **T7 — Documents + IA** | CV, lettres, générateurs, aperçu A4, dépôt PDF, événements Tauri de progression et annulation (§30) | T6 (le profil alimente la génération) |
| **T8 — Réglages** | IA, sauvegardes, mises à jour, à propos ; coffre à secrets | T1 |
| **T9 — Finition** | Accessibilité, responsive 1024→2560, tests, retrait de l'ancien projet Iced | toutes |

Checklist §42 appliquée à chaque tranche avant de la déclarer terminée.

---

## 8. Ce que cet audit n'a pas encore fait

- Lecture ligne à ligne de `app/update/operations/` et `app/update/forms.rs` : des
  enchaînements métier (création de candidature → relance proposée, changement de statut →
  `statut_history`) y sont peut-être encore mêlés à l'orchestration Iced. À faire tranche par
  tranche, pas en bloc.
- Inventaire exhaustif des messages d'erreur utilisateur (`app/message.rs`, 491 lignes).
- Relevé des raccourcis clavier globaux (`app/coquille.rs`).
