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

> **Refonte v2 en cours.** La référence visuelle est désormais `reference_design/`
> (`tokens.json` pour les valeurs, `DECISIONS.md` pour les arbitrages). `styles.css` porte les
> jetons v2 sous leurs noms du handoff (`--bg-app`, `--tx-3`, `--ac`, `--st-g`…) et les
> utilitaires correspondants (`bg-panel`, `text-tx-4`, `bg-ac`, `rounded-r7`, `h-row-app`…).
> Les jetons v1 (`bg-surface`, `text-ink`, `border-line`…) sont **repointés** sur la palette
> v2 le temps que chaque écran soit refondu : un écran neuf ou refondu n'emploie que les
> jetons v2. Les sections ci-dessous marquées « v1 » décrivent les écrans pas encore migrés.

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
- utiliser une police d’affichage (serif, Inter, Geist, etc.) : **system-ui** partout, **JetBrains Mono** seulement pour les identifiants, chemins et valeurs chiffrées ;
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
| `border-accent-border` | Chip actif, item sélectionné **dont les voisins sont sans filet** |
| `border-accent` (plein) | Item sélectionné parmi des voisins **déjà bordés** — voir `ProviderGrid` |
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

Nombres : classe `tabular`. Aucun raccourci clavier de navigation : la coque n’en expose pas, et rien ne les annonce.

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

La police embarquée est une **sous-police** réduite aux icônes de `src/shared/ui/icon-names.ts`, qui est aussi le type `IconName` du composant : une icône hors de cette liste est refusée par `tsc` plutôt qu'affichée en toutes lettres. Ajouter une icône : voir `docs/DEVELOPMENT.md`.

Icônes v2 de la navigation et des listes : `LineIcon` (`src/shared/ui/LineIcon.tsx`), les
11 tracés dessinés pour Candilog (grille 16, trait 1,4 px, `currentColor`) ; marque :
`BrandMark` (tuile `#5B62F0` fixe dans les deux thèmes). Material Symbols reste employé par
les écrans v1 le temps de leur migration.

---

## 7. Copie

- Voix : directe, tutoiement implicite par l’action (« Enregistrer », « Tout effacer »), pas de pitching.
- Un contrôle dit ce qu’il fait. Le toast reprend le même verbe (« Entreprise enregistrée »).
- Erreur : ce qui s’est passé + comment continuer (`ErrorBanner` + Réessayer). Pas d’excuse.
- Vide : titre court + une phrase + une action.
- Dates affichées hors champ : « 02 août » / « 02 août 2026 » (`toLongDate`). Saisie : **`JJ-MM-AAAA`** (`DateInput`, `FORMAT_DATE`). Heure : `HH:MM` (`TimeInput`).
- L’utilisateur n’a pas à connaître Tauri, React, SQLite, le coffre, l’IPC.

---

## 8. Coque v2 (ne pas recréer)

```
┌──────────────────────────────────────────────────────────────┐
│ Barre de titre 40 px : fil d'Ariane · onglets de vue · mention│
├──────────┬───────────────────────────────────────────────────┤
│ Nav      │ main (#contenu) — panneau `bg-panel`, rayon 9      │
│ 202 px   │ (barre d'outils 38 px de l'écran, contenu)        │
│ (52 px   │                                                   │
│ < 1060)  │                                                   │
├──────────┴───────────────────────────────────────────────────┤
│ Barre d'état 34 px : décompte (mono) · contrat clavier       │
└──────────────────────────────────────────────────────────────┘
```

- `AppShell` : fournit le registre de commandes (`shared/lib/commands.tsx`) et le chrome
  (`shared/lib/chrome.tsx`). Les barres ne défilent jamais ; seule la zone de contenu défile.
- `TitleBar` : zone de glissement (`data-tauri-drag-region`). Sous macOS la fenêtre est en
  `titleBarStyle: Overlay` et la barre réserve 68 px aux feux natifs ; sous Windows et Linux
  la barre système est conservée et cette barre se place dessous. Onglets de vue issus de
  `routes.ts` (`DESTINATIONS[].tabs`).
- `Sidebar` : six destinations avec décompte (`useNavCounts`, requêtes rangées sous la clé
  racine de chaque feature), Réglages et indicateur d'IA en pied. `wide:` = ≥ 1060 px.
- `StatusBar` : un écran y écrit son décompte et son contrat clavier par
  `useChrome({ crumb, aside, status, keys })` ; « Actions ⌘K » est toujours présent.
- **Réglages** : surcouche (`SettingsOverlay`, état `settings` du `ui-store`), jamais une
  route. `⌘,` l'ouvre, `Échap` la ferme en rendant l'écran intact.
- **Palette** `⌘K` : `CommandPalette` ; la coque inscrit créer / aller à / réglages
  (`useShellCommands`), chaque écran ajoute ses actions sur la sélection par
  `useRegisterCommands`. Aucune commande qui n'aboutit pas.
- **Raccourcis** : `useShortcut` (`shared/hooks/`) ignore la frappe dans un champ et se tait
  quand une surface est ouverte (`hasOpenSurface`). `G` puis `A/C/R/D/P` change de
  destination. N'inscrire dans Réglages → Raccourcis (`app/overlays/shortcutList.ts`) qu'un
  raccourci réellement câblé. Notation imprimée par `Kbd` / `formatShortcut` : `⌘K` sous
  macOS, `Ctrl K` ailleurs.
- Pas de tour d'accueil (décision D1) : le premier lancement ouvre Aujourd'hui, dont l'état
  vide porte l'amorce.

## 9. Recettes d’écrans

Réutiliser la recette du voisin plutôt que d’en inventer une.

### Relations (Entreprises, Contacts)

- Un seul écran (`features/relations`) : bascule Entreprises / Contacts dans la barre
  d'outils (et non dans la barre de titre), recherche `/`, « + Filtre » (`F`) pour les
  critères fins de la v1 (secteur, type, taille ; rôle), action « Nouvelle entreprise » /
  « Nouveau contact » (`N`).
- Liste groupée, une requête SQLite par groupe : entreprises **En cours** (une candidature
  non refusée), **Repérées** (aucune), **Clôturées** (toutes refusées) ; contacts
  **Recruteurs et managers** (interlocuteurs d'une candidature) et **Réseau**. Lignes de
  38 px : avatar, nom et sous-titre, rattachement, dernière référence, date.
- Inspecteur 300 px (flottant sous 1060 px) : trois actions (Nouvelle candidature, Site web,
  Note ; Écrire, Relancer, Note), champs, candidatures rattachées, historique des faits
  enregistrés, notes. `mailto:` et `tel:` restent des liens natifs ; un lien web passe par
  `openExternal`.

### Documents

- Un seul écran (`DocumentsPage`) : onglets Tous / CV / Lettres / Analyses dans la barre
  d'outils (routes `/documents`, `/documents/resumes`, `/documents/letters`,
  `/documents/analyses`), recherche `/`, actions Importer, Générer une lettre, Générer un
  CV (`N`).
- Liste groupée CV / Lettres de motivation, lignes de 40 px : feuille, nom et sous-titre,
  score ATS en pastille (vert dès 80, accent dès 65, rouge en dessous), date, entreprise.
  Clic droit : Dupliquer ou Copier le texte, Supprimer.
- Inspecteur 300 px (dès 1060 px) : Ouvrir (éditeur A4, qui porte l'aperçu pleine page),
  PDF (`⌘E`), score ATS et constats, détails. Une ancienne version sans contenu structuré
  le dit au lieu d'offrir un export vide.

### Profil

- Colonne de 210 px : complétude (`serif`, segments par section), sections en onglets
  verticaux (`↑ ↓`, `⏎` modifie) avec leur état — disque plein complet, demi-disque partiel,
  cercle vide — et leur décompte, puis « Importer un CV » et « Réinitialiser mon profil ».
- Contenu de la section : titre serif, phrase d'intention, pastille « Complet », puis champs
  (grille 160 px) ou entrées (liseré gauche, période en mono, retrait `×` confirmé) et
  « Ajouter » (`⌘N`). Chaque section s'édite dans son formulaire validé existant.
- La photo se gère dans la section Identité ; Projets et Présence en ligne restent des
  sections à part entière.

### Générateurs (CV, lettre)

- Surcouche plein écran (`GeneratorFrame`) : `×` ou `Échap` ferme (sauf pendant une
  génération), fil « Documents › Générer … », actions à droite, trois colonnes — réglages
  (260 px), feuille A4 sur le bureau `bg-canvas`, déroulé (270 px, dès 1060 px) — et barre
  d'état. `⌘⏎` génère, `⌘S` enregistre : la surcouche porte ses propres raccourcis, ceux
  d'écran se taisant sous une surface.
- « Offre visée » (`OfferSource`) : une candidature ouverte du suivi — son texte est
  prérempli avec ce que Candilog en sait, sans rien inventer — ou le texte collé.
- « Étapes » : les étapes annoncées par le backend, avec la durée **mesurée** de chacune
  (`useStepLog`), jamais estimée.
- Aucun réglage que le backend ne sait pas honorer n'est affiché (pas de sections à
  exclure pour le CV, pas d'arguments autorisés pour la lettre).
- Analyse de CV (`screens/09`) : même surcouche sans colonne droite. À gauche, le CV (PDF)
  « face à » l'offre, puis le score calculé par Candilog et son détail ; au centre, chaque
  exigence de l'offre (`score.evaluations`) — couverte, partielle, absente — avec la preuve
  citée du CV ou « introuvable ». Une exigence manquante sans évaluation détaillée reste
  listée comme absente. Le formulaire est figé, pas masqué, pendant l'analyse.
- Lettre : entreprise, poste, destinataire, ton et longueur restent visibles ; la barre
  « Corrections » sous la feuille envoie une consigne libre ou rapide (« Plus court »…), les
  consignes se cumulent, l'historique vit à droite.

### Candidatures v2 (Liste, Kanban)

```
Barre d'outils 38 px : puces · + Filtre F · Tout effacer ……… n / total · Rechercher / · CSV · Ajouter N
contenu (liste groupée par statut ou Kanban) + inspecteur 380 px (flottant sous 1060 px)
barre groupée 40 px dès qu'une case est cochée
```

- **Puces** (`ApplicationToolbar`) : une par critère, « Champ est / n'est pas Valeurs ».
  Un clic inverse la condition (`excluded` côté backend), la croix retire le critère. Les
  bornes (heures, période d'envoi) ne s'inversent pas.
- **« + Filtre »** (`ApplicationFilterMenu`, sur `Menu`) : deux niveaux, champ puis valeur ;
  `←` revient aux champs. Tous les critères backend y figurent, y compris les critères fins
  de la v1 (domaine, type et taille d'entreprise, secteur, régime, heures, poste, ville,
  période). Une saisie invalide est signalée dans le menu et n'est jamais appliquée.
- Recherche : `/` y place le focus, `Échap` l'efface puis la quitte.
- **Barre groupée** (`BulkBar`) : changer le statut (`S`), exporter en CSV (`⌘E`),
  supprimer (`⌘⌫`), désélectionner (`Échap`). Elle agit sur les cases cochées, jamais sur
  la seule fiche ouverte.
- **Kanban** : colonnes élastiques 236–340 px sur `bg-group`, défilement horizontal ;
  bandeaux « sans réponse depuis plus de 14 jours » et « entretien aujourd'hui » ; cartes
  compactes (référence, échéance, intitulé, entreprise, contrat, date) ; chaque colonne est
  paginée côté SQLite (`ColumnPager`). Un changement de statut affiche `CAN-142 → Entretien`.

### Analyse

- Barre d'outils : période (30 j, 90 j, tout) et « Exporter en CSV ».
- Quatre indicateurs (envoyées, taux de réponse, entretiens, délai moyen) sans flèche de
  tendance : il n'existe pas de période de comparaison, une variation serait inventée.
- Blocs sur `bg-group` : parcours des candidatures (part et perte à chaque étape), rythme
  d'envoi, taux de réponse par canal (`Analytics.channels`, même définition d'une réponse
  que les indicateurs), « Ce que disent ces chiffres » (`model/insights.ts` : constats
  vérifiables dans les blocs voisins, jamais de projection), candidatures à relancer,
  performance.

### Calendrier

- Barre d'outils : `‹ Mois ›` en serif, « Aujourd'hui », légende avec les décomptes, vues
  Mois / Semaine / Jour (décision D6), « Programmer une relance », « Nouvel entretien ».
- Grille plate à filets `bd-soft`, aujourd'hui sur `bg-sel` avec son numéro en pastille
  `ac`. Pastilles (`eventStyle`) : l'entreprise d'abord, l'heure en mono ; entretien vert,
  relance à venir neutre à glyphe ambre, relance en retard rouge, relance faite atténuée.
  Deux pastilles par case, puis « +N ».

### Aujourd’hui

- Bloc centré de 1 180 px : titre serif de la date, résumé mono, puis trois horizons (en
  retard, aujourd'hui, cette semaine) sur `bg-group` ; lignes de 44 px (34 px pour la
  semaine). `⏎` fait la relance focalisée, `R` la reporte.
- Colonne Situation de 290 px (dès 1060 px) : répartition des statuts, 30 derniers jours,
  candidatures sans réponse.
- Base neuve : « Votre suivi commence ici » ; rien de dû : « Rien à faire aujourd'hui ».

### Graphiques (Aujourd’hui, Analyses)

- Une seule bibliothèque : **Recharts**, en SVG, dans `analytics/view/components/charts`.
- Couleurs prises dans `chartTheme.ts`, qui ne contient que des `var(--candilog-…)` :
  le SVG résout la variable au rendu, un changement de thème repeint donc sans re-render.
- Aucune valeur accessible par le seul survol : axe visible ou liste `sr-only` équivalente.
- Une seule série → pas de légende, la carte la nomme. Deux séries ou plus → légende
  systématique, avec libellé **et** compte, car les teintes de statut vert et rouge sont
  proches pour une deutéranopie.
- États vides gérés par le graphique lui-même (`EmptyState`), pas par l’écran appelant.

### Réglages (IA, Sauvegardes, Mises à jour, À propos)

- `PageHeader` + `SettingsBody` (padding 18 / 16 / 22, gap 4, scroll).
- Colonne de contenu **max 720 px** quand c’est une fiche (À propos).
- `SettingsCard` : en-tête à filet, icône tertiaire 17 px, titre `text-item`.
- `ActionCard` : **une action** (export, rechercher une MAJ) — pas une grille de bénéfices produit.
- `SettingsHero` : écrans de **maintenance** (version, sauvegarde), pas un slogan.
- À propos : identité (logo + nom + version) + faits (`InspectorRow`) + auteur. **Pas** de hero, **pas** de pile technique.
- IA (`screens/12`, `13`) : colonne **Fournisseurs** de 190 px (onglets verticaux : état « Local · n modèles », « Clé enregistrée », « Aucune clé », point vert quand le fournisseur est prêt, mention « principal »), puis le détail : nom en serif, description **factuelle** et trois jauges — confidentialité, coût, hors connexion ; jamais une promesse de qualité. L'IA locale affiche son catalogue (`ManagedOllamaPanel`), un fournisseur distant sa configuration (modèle et `RemoteModelPicker` après Actualiser, endpoint, clé jamais rendue en clair, mode, température) avec « Tester la connexion » (`T`) et « Enregistrer », qui en fait le fournisseur principal. En bas, **Qui fait quoi** (`AiTaskRouting`, `reference_design/AI_TASK_ROUTING.md`) : cinq tâches, le modèle de chacune, point vert (local), ambre (distant) ou gris (désactivée) ; le sélecteur propose le fournisseur principal, les modèles installés, les fournisseurs distants configurés et « Aucun » ; le choix est enregistré aussitôt, sans repli. Le fournisseur local s’appelle **« IA locale »**, jamais d’après une famille de modèles. La liste **Profils disponibles** reprend les `evaluations` du backend (`compatibility` + `reason`) : un profil `unsupported` est étiqueté « Incompatible » et son installation est désactivée. Un benchmark `too_slow` avertit **toujours**, y compris sur le plus petit profil où aucun repli n’existe. Le résultat d'un test de connexion est un message fixe : la prose du modèle n’est pas un état.
- Mises à jour : colonne bornée à 760 px, une carte de surface unique — vignette d’état, phrase, pastille et action en tête, puis les versions sous un filet, puis la progression. Les notes de version, quand il y en a, forment une `SettingsCard` `Nouveautés`. Rien sur le mécanisme de téléchargement — cela n’aide pas à décider.

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
| Traitement IA en cours | `AiProgress` : étape, barre indéterminée et temps écoulé — **jamais** de pourcentage, il serait inventé |
| Traitement IA terminé | Durée totale en badge d'en-tête (« Rédigée en 12 s ») |

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
| Graphique | `analytics/view/components/charts` (Recharts) — jamais des `div` à largeur calculée |
| Surface glass overlay | classes `glass-popover`, `glass-modal` |

`Card` existe pour des blocs denses déjà dans le design ; ne pas s’en servir pour recréer un dashboard de widgets.

---

## 11. Glass et overlays

La coque est vitreuse (`backdrop-filter` 16–20 px). Les **overlays** (popover Filtres adaptatif de 230 à 640 px selon son contenu, menus, modale) utilisent `glass-popover` / `glass-menu` / `glass-modal` et `shadow-overlay` / `shadow-menu`.

Sans `backdrop-filter`, fallback `glass-fallback` / `surface-elevated` (déjà dans `styles.css`).

Fermeture : `useDismissable` (Escape + clic extérieur) — calendrier, FilterMenu, inspecteur, modale.


---

### Documents générés

#### CV ciblé (éditeur)

- L'aperçu est une **feuille A4 unique** (`ResumePaper`, 210 × 297 mm) rendue en HTML avec
  les jetons `--resume-*` de `styles.css` : encre, accent, filets, marges (14 / 16 / 15 mm)
  et géométrie `--resume-page-*`. Le papier reste **blanc** (`--paper-bg`) en thème clair
  **et** sombre — il prévisualise le PDF imprimé, pas la surface de l'application.
- Typographie **IBM Plex Sans** (corps) et **IBM Plex Mono** (étiquettes, dates, coordonnées),
  polices locales embarquées (`src-tauri/assets/fonts/ibm-plex/`), identiques à l'export PDF.
- Cinq **paliers de densité** (`--resume-fs`, `--resume-sp`) : `ResumePaper` compacte
  d'abord les espacements, puis la taille de corps jusqu'au seuil lisible minimal. Si le
  contenu dépasse encore la hauteur imprimable, un bandeau `resume-overflow-warning` l'indique
  et le bouton **Exporter** est désactivé — aucun texte n'est tronqué silencieusement.
- L'édition est **directe sur le papier** (`ResumeEditableText`) : le texte affiché est celui
  qui sera enregistré et exporté. Le collage ne conserve que le texte brut.
- Le panneau latéral **Aide au contenu** sépare clairement « Recommandé pour cette offre »
  (petite sélection expliquée) de « Disponible dans votre profil » (bibliothèque complète
  absente du document). Les listes sont compactes, sans score IA artificiel ni accumulation
  de cards. Ajouter ou ignorer met le panneau et le papier à jour immédiatement.
- Un indicateur qualitatif reprend la mesure PDF : Bonne marge, Espace disponible, Peu
  d'espace restant, CV presque plein ou Dépassement. Sa jauge utilise la hauteur mesurée,
  mais aucun faux pourcentage n'est affiché. En dépassement, les choix restent intacts et
  les recommandations cessent d'aggraver la page.
- Les compétences demandées mais absentes du profil apparaissent dans « Compétences
  manquantes à vérifier », sans bouton qui les ferait passer pour acquises.
- Une fois le CV généré, le panneau **Offre ciblée** s'efface : à trois colonnes l'aperçu
  était trop étroit pour une page A4. **Modifier l'offre** le ramène, **Revenir au CV** le
  referme. Les panneaux défilent, et le papier ne se comprime jamais sous sa largeur A4.

- L'aperçu A4 de la lettre (`LetterPaper`, 210 × 297 mm) reprend le template HTML fourni :
  colonne d'identité 58 mm (`--letter-panel`) et corps à droite, jetons `--letter-*`,
  typographie **IBM Plex Sans** / **IBM Plex Mono**. Le papier reste blanc dans les deux
  thèmes. L'identité (nom, titre, adresse, ville, téléphone, courriel) vient du **profil
  courant** et s'édite directement sur la feuille : la sortie du champ enregistre le profil,
  jamais la frappe. Elle reste en lecture tant que le profil n'est pas chargé, sinon la
  saisie serait perdue. Entreprise, poste, interlocuteur, adresse destinataire et référence
  d'offre s'éditent aussi sur la feuille, mais sont enregistrés avec la lettre. Les blocs
  vides sont omis en lecture. « Pièce jointe : curriculum vitæ » est toujours affiché.
- Le PDF reprend les cotes du template en pixels CSS (`pt(px)`) et son `letter-spacing`,
  faute de quoi l'aperçu et la page imprimée divergent. Tout bloc de la colonne d'identité
  se replie dans les 58 mm ; un titre long y passe sur plusieurs lignes plutôt que de
  déborder sur la lettre. Un **mot** plus large que la colonne — un patronyme composé — est
  coupé : ne pas couper les mots est une préférence de composition, pas une autorisation à
  sortir du cadre.
- Quatre **paliers de densité** (`--letter-fs`, `--letter-sp`) compactent la feuille si le
  texte déborde. Au-delà, un bandeau `letter-overflow-warning` l'indique et **Exporter** /
  **Enregistrer** sont désactivés.
- L'aperçu A4 de la lettre est **éditable sur place** : le texte affiché est celui qui sera
  enregistré et exporté.
- Barre d'outils de la lettre : gras, souligné, taille (petite / normale / grande) et
  alignement. Rien d'autre — un bouton dont l'effet disparaîtrait à l'export serait un
  piège, et les polices embarquées n'ont pas d'italique.
- Une fois la lettre écrite, le brief laisse la place au bloc **Itérations** : consignes
  cumulées, durée de chaque régénération, retour au brief possible.
- Les champs d'offre et de contexte portent un bouton « Coller » (lecture native du
  presse-papiers), en plus du Ctrl+V habituel.

---

## Documents PDF

- CV : exactement une page A4 (210 × 297 mm), texte sélectionnable, polices IBM Plex
  embarquées. Le moteur Rust (`infrastructure/pdf/resume_pdf.rs`) reproduit la même logique
  de densité que l'aperçu : espacements puis typographie jusqu'au seuil lisible. Le libellé
  de section se replie dans sa colonne (`LABEL_W`) plutôt que de déborder sur le contenu.
- **Aperçu et export partagent la même géométrie**, jusqu'au détail :
  - l'interlignage suit la **police** (`line-height` en multiple du corps), l'échelle de
    densité ne compresse que les **écarts entre blocs**. Le moteur PDF appliquait l'échelle
    d'espacement à ses interlignes : au palier le plus dense il tassait ses lignes de 24 %,
    et un CV que l'aperçu déclarait trop long s'exportait malgré tout, dans une mise en page
    que l'utilisateur n'avait jamais vue ;
  - l'**interlettrage** du gabarit (nom, sous-titre, étiquettes, périodes) est appliqué à
    l'export comme à l'écran, sans quoi les mêmes libellés sortaient plus étroits ;
  - la colonne d'étiquettes (`LABEL_W`, `grid-cols-[116px_1fr]`) est dimensionnée pour le
    plus long libellé du gabarit à son interlettrage réel.
- **Aucun champ n'est posé sur une ligne unique** : intitulé, entreprise, diplôme,
  établissement, projet, coordonnées et langues se replient dans leur colonne, et un mot
  plus large qu'elle est coupé. Sans ce repli, un seul nom d'employeur un peu long faisait
  refuser tout l'export — au motif, en plus, que le CV serait « trop long ».
- Le refus nomme sa cause : un CV trop **large** n'est pas un CV trop **long**, et le
  raccourcir n'y changerait rien.
- Les deux moteurs remplacent par une espace tout caractère absent des polices embarquées —
  un retour à la ligne saisi dans un champ mono-ligne, par exemple — qui sortait sinon en
  rectangle vide alors que l'aperçu HTML le rendait correctement.
- Lettre de motivation : exactement une page A4 (210 × 297 mm).
- Le rendu réduit d'abord les espacements, puis la typographie jusqu'au seuil lisible défini par le moteur.
- Si le contenu ne tient toujours pas, l'export est refusé avec un message demandant de le raccourcir. Aucun texte n'est tronqué, superposé ou placé hors page silencieusement.

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
