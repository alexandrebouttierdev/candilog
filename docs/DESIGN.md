# Design system Candilog

Source de vérité visuelle pour toute interface. **Lire ce fichier avant de créer ou modifier un écran.**

Candilog est une **application desktop de productivité** (Tauri), pas un dashboard SaaS. La hiérarchie vient des surfaces, des filets et de l’espacement — pas des ombres, des cartes marketing ni des dégradés.

Sources dans le code :

| Quoi | Où |
| --- | --- |
| Jetons, thèmes, typo, rayons | `src/styles.css` |
| Composants | `src/shared/ui/` (réexporter via `index.ts`) |
| Coque | `src/app/layout/` |
| Routes et icônes de nav | `src/app/router/routes.ts` |
| Galerie | `src/app/dev/DesignGallery.tsx` |

Ne pas inventer de couleur, de rayon, de gabarit de bouton ou de composant déjà présent dans `shared/ui`.

---

## 1. Produit

**Sujet.** Un suivi de recherche d’emploi : candidatures, réseau, documents, tout **sur cet appareil**.

**Public.** Une personne qui travaille dans Candilog tous les jours, au clavier, sur une fenêtre native.

**Job d’un écran.** Faire une tâche précise (filtrer, ouvrir une fiche, enregistrer) — jamais vendre le produit.

L’interface est en **français**. Les identifiants de code sont en **anglais** (`snake_case` IPC / Rust / SQL).

---

## 2. Interdits (causes de divergence)

Ne pas :

- ressembler à un dashboard web (grosses cards, KPI en héros, dégradés, blobs) ;
- poser des hexadécimaux ou des `rgb()` dans un composant — uniquement les classes Tailwind du thème (`bg-surface`, `text-ink`, `border-line`, `text-accent`, …) ;
- changer l’accent (indigo `#4f5fe8` / `#6b7cff` en sombre) ni introduire un second accent (orange, vert néon, terracotta) ;
- utiliser une police d’affichage (serif, Inter, Geist, etc.) : **system-ui** partout, **JetBrains Mono** seulement pour `kbd`, identifiants, chemins ;
- multiplier les ombres : `shadow-1` et `shadow-accent` sont `none` ; l’ombre n’existe que sur overlays (modale, menu, palette) ;
- agrandir les rayons (pas de `rounded-2xl` / `rounded-3xl` décoratifs) ;
- recréer un bouton, un champ, une pastille, une barre de filtres ou une modale « pour cet écran » ;
- mettre la recherche d’une liste paginée dans la topbar (`ContextSearch`) si l’écran a déjà une `FilterBar` (Candidatures, Entreprises, Réseau) ;
- exposer la pile technique à l’utilisateur (Tauri, React, SQLite, IPC…) ;
- écrire un slogan ou un hero marketing (À propos n’est **pas** une landing) ;
- laisser un état vide sans issue (action ou `Tout effacer`) ;
- porter l’information par la couleur seule : toute pastille a un libellé.

---

## 3. Couleur

Thème **clair par défaut**, sombre via `data-theme` / préférence système. Les classes Tailwind mappent les CSS variables.

### Surfaces

| Classe | Rôle |
| --- | --- |
| `bg-page` | Fond de fenêtre `#f2f3f6` / `#08090c` |
| `bg-surface` | Panneau, liste, carte à filet |
| `bg-surface-alt` | Pied de liste, pied de modale |
| `bg-surface-elevated` | Overlay sans glass |
| `bg-fill` / `hover:bg-fill-hover` | Contrôle au repos |
| `bg-accent-tint` / `accent-tint-08` / `accent-tint-12` | Sélection, pastille, item actif |

### Filets

| Classe | Rôle |
| --- | --- |
| `border-line` | Séparation de panneau |
| `border-line-soft` | Filet d’écran (header, FilterBar) |
| `border-control` / `border-control-strong` | Bouton, champ, trigger |
| `border-accent-border` | Chip actif, item sélectionné |
| `border-field` | Rangée d’inspecteur |

La hiérarchie = **filet 1 px** + contraste de surface. Pas de drop-shadow sur les cartes de contenu.

### Texte

| Classe | Usage |
| --- | --- |
| `text-ink` | Titre, corps principal |
| `text-ink-strong` | Valeur d’inspecteur |
| `text-ink-muted` / `text-ink-tertiary` | Secondaire |
| `text-ink-faint` / `text-ink-subtle` | Meta, placeholder, icône tertiaire |
| `text-ink-label` | Eyebrow de sous-nav (`uppercase`) |
| `text-accent` / `text-accent-text` / `text-accent-text-soft` | Accent lisible (pas le bleu brut sur fond blanc en long texte) |
| `text-on-accent` | Sur bouton primary |
| `text-success` / `text-warning` / `text-danger` | Statut sémantique |

### Sémantique (`Tone`)

Vert = avancement · ambre = à traiter · rouge = échec · accent = mis en avant · neutre = attente. Composant : `StatusPill`.

---

## 4. Typographie

`font-sans` = system-ui. Corps de page : `text-body` (12,5 px), `letter-spacing: 0.005em`.

| Classe | Taille | Poids | Où |
| --- | --- | --- | --- |
| `text-display` | 23 px | 600 | Rare (chiffre fort, pas un titre de page) |
| `text-heading` / `text-kpi` | 18 px | 600 | KPI, titre de modale |
| `text-title` | 14,5 px | 600 | Identité d’écran (nom produit, fournisseur) |
| `text-section` | 13,5 px | 600 | `PageHeader` h1, nom d’auteur |
| `text-item` | 13 px | 600 | Bouton, titre de carte, ligne de liste |
| `text-body` | 12,5 px | 400 | Corps, item de sous-nav |
| `text-note` | 12 px | — | Sous-titre, total de FilterBar |
| `text-label` | 11,5 px | — | Chip, aide, compteur de liste |
| `text-meta` | 11 px | — | Erreur sous champ, note |
| `text-eyebrow` | 10,5 px | 600, tracking 0.07em, uppercase | Labels de groupe (Filtres, sous-nav) |

Nombres : classe `tabular`. Raccourcis : classe `kbd` (JetBrains Mono 10,5 px).

---

## 5. Densité et rayons

Candilog est **dense**. Contrôles à **30 px** (`h-control`). Topbar **46 px**. Rail **68 px**. Sous-nav **186 px**.

| Jeton | Valeur | Usage |
| --- | --- | --- |
| `rounded-card` | 12 px | Carte à filet (`SettingsCard`) |
| `rounded-tile` | 10 px | Tuile, pastille d’empty state |
| `rounded-field` | 9 px | Champ formulaire |
| `rounded-button` / `rounded-control` | 8 px | Bouton, item de sous-nav, FilterTrigger |
| `rounded-chip` | 7 px | Chip, option de filtre |
| `rounded-pill` | 6 px | StatusPill |
| `rounded-overlay` | 14 px | Modale, popover Filtres, confirmation |

Hover : `duration-hover` (120 ms), couleur seulement. `prefers-reduced-motion` : pas d’entrée de palette.

Focus clavier : `outline 1px accent-focus`, offset 0 — déjà sur `:focus-visible` global.

---

## 6. Icônes

**Material Symbols Rounded**, police locale, `wght` 300, jamais FILL sauf `filled`.

Composant : `Icon` (`src/shared/ui/Icon.tsx`). Tailles usuelles : 14 (pastille), 15–16 (bouton, nav), 17 (header de carte), 20 (empty).

Noms d’icônes des sections : `src/app/router/routes.ts`. Logo produit : `src/assets/logo-candilog.svg` (36×36 dans le rail ; variante sombre `logo-candilog-dark.svg` ; coche `#4FC27A`).

---

## 7. Copie

- Voix : directe, tutoiement implicite par l’action (« Enregistrer », « Tout effacer »), pas de pitching.
- Un contrôle dit ce qu’il fait. Le toast reprend le même verbe (« Entreprise enregistrée »).
- Erreur : ce qui s’est passé + comment continuer (`ErrorBanner` + Réessayer). Pas d’excuse.
- Vide : titre court + une phrase + une action.
- Dates affichées hors champ : « 02 août » / « 02 août 2026 » (`versDateLongue`). Saisie : **`JJ-MM-AAAA`** (`DateInput`, `FORMAT_DATE`). Heure : `HH:MM` (`TimeInput`).
- L’utilisateur n’a pas à connaître Tauri, React, SQLite, le coffre, l’IPC.

---

## 8. Coque (ne pas recréer)

```
┌──────┬──────────────────────────────────────────────┐
│ Rail │ Topbar (titre de section + ⌘K)               │
│ 68px │──────────────────────────────────────────────│
│      │ SubNav 186px │  main  (#contenu)             │
│      │ (si >1 route)│                               │
└──────┴──────────────┴───────────────────────────────┘
```

- `AppShell` : `h-screen overflow-hidden`, glass sur rail / topbar / sous-nav (`glass-rail`, `glass-topbar`, `glass-subnav`).
- `NavRail` : 7 sections, `⌘1`…`⌘7`, tooltip = `long_label`, item actif en teinte accent.
- `SubNav` : eyebrow = `short_label` uppercase ; item 30 px ; actif `bg-accent-tint-12 text-accent-text-soft`.
- `TopBar` : titre de section. Accessoire **à droite** via `ContextBarAccessory` (note, ou recherche **seulement** si l’écran n’a pas de FilterBar).
- Palette de commandes : `CommandPalette`, déclencheur « Rechercher ou exécuter… ⌘K ».

Le workspace (`main`) est un **outil plein cadre** : header d’écran + contenu, sans padding de page type site web (sauf Réglages, voir §10).

---

## 9. Recettes d’écrans

Réutiliser la recette du voisin plutôt que d’en inventer une.

### Liste filtrée (Candidatures, Entreprises, Réseau)

```
PageHeader (titre + sous-titre, sans search ni primary si la barre les porte)
FilterBar : SearchInput toolbar 300px · Filtres · chips · Tout effacer · {n} nom(s) · actions à droite
contenu (Kanban/table ou MasterList + fiche)
```

- `SearchInput variant="toolbar"` : placeholder **« Rechercher… »**.
- `FilterMenu` + `FilterGroup` + `FilterOption` (pastilles, `aria-pressed`).
- Un critère = un `ActiveFilterChip` « Champ · Valeur ».
- `filtersActifs` **exclut** la recherche libre (pastille du bouton Filtres).
- La recherche et les filtres sont des **paramètres de requête backend**, jamais un `.filter()` sur la page affichée.
- Action primaire (« Nouvelle », « Nouveau contact ») **dans** `FilterBar.actions`, pas dans le `PageHeader`.
- Vide + critères : « Aucun résultat » + bouton Tout effacer.

Références : `ApplicationFilters`, `CompanyFilters`, `ContactFilters`.

### Maître-détail (Entreprises, Réseau)

- `MasterList` : 37 % largeur, min 300 px, `bg-surface`, filet droit.
- Item : `MasterListItem` (initiales, titre, sous-titre, `MasterListTag`).
- Fiche à droite ; si rien de sélectionné et liste non vide, ouvrir le premier item.
- Pagination : `Pager` dense dans le pied de liste.

### Table / Kanban (Candidatures)

- `FilterBar` identique.
- Liste : `DataTable` + `CellIdentity` + `StatusPill` + `Pager`.
- Kanban : colonnes denses, sélection multiple → actions dans la FilterBar (pas une barre flottante SaaS).
- Fiche : `Inspector` (380 px, redimensionnable 320–460, glass), rangées `InspectorRow`.

### Réglages (IA, Sauvegardes, Mises à jour, À propos)

- `PageHeader` + `SettingsBody` (padding 18 / 16 / 22, gap 4, scroll).
- Colonne de contenu **max 720 px** quand c’est une fiche (À propos).
- `SettingsCard` : en-tête à filet, icône tertiaire 17 px, titre `text-item`.
- `ActionCard` : **une action** (export, rechercher une MAJ) — pas une grille de bénéfices produit.
- `SettingsHero` : écrans de **maintenance** (version, sauvegarde), pas un slogan.
- À propos : identité (logo + nom + version) + faits (`InspectorRow`) + auteur. **Pas** de hero, **pas** de pile technique.

### Formulaires

- Toujours `ModalHost` (620 px par défaut, overlay `rounded-overlay`, pied fixe visible).
- Champs : `FormField` + `TextInput` / `Select` / `TextArea` / `DateInput` / `TimeInput` / `EntityPicker`.
- Erreur **sous** le champ (`aria-invalid`, `aria-describedby`), jamais une infobulle seule.
- Requis : astérisque danger sur le libellé.

### Destruction

- `ConfirmDialog` (440 px). Titre en question. Description : ce qui disparaît. `note` : ce qui survit.
- Confirm = `danger`. Pas de toast à la place d’une confirmation.

### Feedback

| Situation | Composant |
| --- | --- |
| Chargement d’écran | `Skeleton` / `SkeletonRows`, `role="status"` |
| Échec de chargement | `ErrorBanner` + Réessayer |
| Succès / échec d’écriture sans décision | `notify()` → `Toaster` (4 s, bas droite) |
| Décision destructive | `ConfirmDialog` |
| Rien à montrer | `EmptyState` dans le contenant, pas un écran plein décoratif |

---

## 10. Composants — quand les prendre

Toujours importer depuis `@/shared/ui` (sauf `SettingsUi`, propre aux réglages).

| Besoin | Composant |
| --- | --- |
| Action | `Button` (`primary` \| `secondary` \| `ghost` \| `danger`), `h-control` 30 px |
| Icône seule | `IconButton` 30×30, `aria-label` obligatoire |
| Recherche d’outil | `SearchInput variant="toolbar"` dans une `FilterBar` |
| Recherche topbar (docs, etc.) | `ContextSearch` via `ContextBarAccessory` |
| Filtres d’une liste | `FilterBar` + `FilterMenu` + chips |
| Titre d’écran | `PageHeader` (h1 `text-section`) |
| Bascule Kanban/Liste | `SegmentedControl` |
| Statut | `StatusPill` + `Tone` |
| Attribut sans statut | `Tag` |
| Liste maître | `MasterList` / `MasterListItem` / `MasterListTag` |
| Tableau | `DataTable` |
| Fiche latérale | `Inspector` + `InspectorRow` + `InspectorSectionLabel` |
| Split redimensionnable | `SplitPane` / `TripleSplitPane` |
| Modale métier | `ModalHost` |
| Date / heure | `DateInput` / `TimeInput` (saisie **ou** picker, format FR) |
| Pagination | `Pager` / `ColumnPager` |
| KPI compact (Analyses) | `StatCard` — pas en bandeau de chaque écran |
| Surface glass overlay | classes `glass-popover`, `glass-modal`, `glass-palette` |

`Card` existe pour des blocs denses déjà dans le design ; ne pas s’en servir pour recréer un dashboard de widgets.

---

## 11. Glass et overlays

La coque est vitreuse (`backdrop-filter` 16–20 px). Les **overlays** (popover Filtres 230 px, menus, palette, modale) utilisent `glass-popover` / `glass-menu` / `glass-modal` / `glass-palette` et `shadow-overlay` / `shadow-menu`.

Sans `backdrop-filter`, fallback `glass-fallback` / `surface-elevated` (déjà dans `styles.css`).

Fermeture : `useDismissable` (Escape + clic extérieur) — calendrier, FilterMenu, inspecteur, modale.

---

## 12. Accessibilité (plancher)

- Un `h1` par écran (`PageHeader`).
- Focus visible global. Pas de `outline-none` sans remplacement.
- Bouton Filtres : `aria-label` « Filtres » ou « Filtres, n actifs ».
- Chips : `aria-label` « Retirer le filtre … ».
- Empty / loading : `role="status"` ou `alert` selon le cas.
- Contraste : ne pas poser `text-accent` (bleu saturé) sur de longs paragraphes ; préférer `text-accent-text`.
- `prefers-reduced-motion` respecté (palette).

---

## 13. Checklist agent

Avant de merger un changement d’UI :

1. J’ai réutilisé un composant de `shared/ui` plutôt que d’en créer un visuellement proche.
2. Aucun hex / `rgb()` nouveau dans le TSX.
3. Contrôles à 30 px, filets 1 px, pas d’ombre sur le contenu.
4. Libellés français, identifiants anglais.
5. Liste paginée : filtre et recherche côté backend + FilterBar (si Candidatures / Relations).
6. Pas de pile technique ni de hero marketing.
7. États vide, erreur, chargement traités.
8. Vérifié clair **et** sombre (classes sémantiques, pas de `bg-white`).

En cas de doute, copier **Candidatures** (outil), **Entreprises** (maître-détail) ou **À propos** (fiche réglages) — pas un template externe.
