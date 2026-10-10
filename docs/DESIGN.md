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

**La référence visuelle est `src/styles.css`**, qui porte les jetons sous leurs noms du
handoff (`--bg-app`, `--tx-3`, `--ac`, `--st-g`…) et les expose en utilitaires Tailwind
(`bg-panel`, `text-tx-4`, `bg-ac`, `rounded-r7`, `h-row-app`…). Une valeur se lit là, et le
présent document dit comment l'employer.

> **Notation du handoff.** Le dossier de livraison du design (`tokens.json`,
> `DECISIONS.md`, `INTERACTIONS.md`, `PLATFORM.md`, `AI_TASK_ROUTING.md`, `screens/`,
> `states/`) **ne fait pas partie du dépôt**. Les commentaires du code y renvoient encore par
> leur notation courte — `DECISIONS.md D7`, `INTERACTIONS.md §3.2`, `screens/15`,
> `states/dialog-stop-gen.png`. Ce sont des **références de provenance**, pas des liens :
> elles disent d'où vient un arbitrage, non où le relire. Ce qui en survit et fait autorité
> est ici, dans `docs/AI.md` (routage des tâches, gardes d'envoi distant D3/D4/D13) et dans
> `src/styles.css`. Ne pas créer de règle nouvelle qui dépende d'un fichier du handoff : elle
> serait inapplicable.

> **Jetons v1 hérités.** `bg-surface`, `text-ink`, `border-line`, `text-accent`,
> `rounded-card`, `text-body`… existent encore : ils sont **repointés** sur la palette v2
> (table en §3) et une trentaine de composants les emploient toujours. Tout code neuf ou
> retouché n'emploie que les jetons v2 ; un composant qu'on modifie en profondeur passe
> aux jetons v2 au passage.

---

## 1. Produit

**Sujet.** Un suivi de recherche d’emploi : candidatures, réseau, documents, tout **sur cet appareil**.

**Public.** Une personne qui travaille dans Candilog tous les jours, au clavier, sur une fenêtre native.

**Job d’un écran.** Faire une tâche précise (filtrer, ouvrir une fiche, enregistrer) — jamais vendre le produit.

L’interface est en **français**. Les identifiants de code sont en **anglais** (`snake_case` IPC / Rust / SQL).

---

## 2. Interdits (causes de divergence)

Ne pas :

- ressembler à un dashboard web (grosses cartes, KPI en héros, dégradés, blobs) ;
- poser des hexadécimaux ou des `rgb()` dans un composant — uniquement les utilitaires du
  thème (`bg-panel`, `text-tx-3`, `border-bd`, `bg-ac`…) ; seule exception, la tuile de
  marque `bg-brand` ;
- changer l’accent (indigo `#4A51DF`, `#5B62F0` en sombre) ni introduire un second accent ;
- utiliser une autre police que les trois du design : **system-ui** pour l’interface,
  **IBM Plex Serif** pour les titres d’écran, de dialogue et les scores (≥ 15 px,
  `serif-title`), **IBM Plex Mono** pour les références (`CAN-142`), dates courtes, durées,
  compteurs et touches ;
- multiplier les ombres : une seule (`shadow-pop`), réservée aux menus, dialogues et
  formulaires modaux ; `shadow-sheet` pour la feuille A4 des aperçus ;
- agrandir les rayons au-delà de `rounded-r11` (fenêtre modale) — deux jetons v1 le
  dépassent encore et sont de la dette, pas une permission : `rounded-card` (12 px,
  19 emplois) et `rounded-overlay` (14 px, `DateInput`). Ne pas en introduire d'autres,
  et passer un composant retouché à l'échelle `r2`–`r11` ;
- ajouter une police d’icônes ou une bibliothèque de graphiques (décision D7, §6) ;
- recréer un bouton, un champ, une pastille, une barre de filtres ou une modale « pour cet écran » ;
- exposer la pile technique à l’utilisateur (Tauri, React, SQLite, IPC…) ;
- écrire un slogan ou un hero marketing (À propos n’est **pas** une landing) ;
- laisser un état vide sans issue (action ou `Tout effacer`) ;
- porter l’information par la couleur seule : un statut a un libellé et un `StatusGlyph`,
  dont la forme (vide, demi, trois quarts, plein) se lit sans couleur.

---

## 3. Couleur

Thème **clair par défaut** (gris froid, même famille que le sombre), sombre par
`data-theme="dark"` ou le mode
« Système » (`prefers-color-scheme`). Chaque jeton existe dans les deux thèmes ; un
composant n’a jamais de variante `dark:` de couleur.

### Surfaces

| Utilitaire | Rôle |
| --- | --- |
| `bg-app` | Fond de fenêtre, navigation, barres de titre et d’état |
| `bg-panel` | Zone de travail : liste, fiche, inspecteur |
| `bg-group` | Bloc dans un panneau, en-tête de groupe, section de réglage |
| `bg-chip` | Pastille, touche, piste de jauge, barre à zéro |
| `bg-elev` | Contrôle segmenté, surface surélevée |
| `bg-hover` / `bg-sel` | Survol / sélection d’une ligne (`row-focus`, `row-selected`) |
| `bg-modal` / `bg-menu` / `bg-input` | Dialogue / menu / champ |
| `bg-canvas` | Bureau sous la feuille A4 des générateurs |

### Filets

`border-bd` sépare deux panneaux ; `border-bd-soft` sépare deux lignes ou une barre d’outils
de son contenu ; `border-bd-menu` borde un menu, un champ, un contrôle. La hiérarchie vient
du contraste de surface et du filet 1 px, jamais d’une ombre.

### Texte

Sept encres, de la plus forte à la plus faible :

| Utilitaire | Usage |
| --- | --- |
| `text-tx` | Titre, valeur, ligne sélectionnée |
| `text-tx-2` / `text-tx-3` | Corps, libellé de ligne |
| `text-tx-4` / `text-tx-5` | Secondaire, méta, compteur |
| `text-tx-6` | En-tête `caps`, placeholder, désactivé |
| `text-tx-7` | Filigrane, séparateur typographique |
| `text-ac-tx` | Lien, accent lisible sur fond clair |

### Accent, statuts, teintes

- `bg-ac` : action primaire, barre de graphique, élément actif ; `text-ac-tx` pour le texte ;
  `bg-ac-soft` pour une surface accentuée.
- `st-n` · `st-a` · `st-g` · `st-c` : neutre (attente), ambre (à traiter), vert
  (avancement), rouge (échec). Portés par `StatusGlyph` ; jamais un aplat de fond.
- Teintes de pastille, fond et encre appariés : `bg-tint-ac-bg text-tint-ac-tx`,
  `bg-tint-g-bg text-tint-g-tx`, `bg-tint-c-bg text-tint-c-tx`.
- La croix de fermeture d'une surcouche porte la teinte `bg-tint-c-bg text-tint-c-tx` :
  seule sortie d'une surface plein écran, elle doit se repérer d'un coup d'œil. C'est la
  seule exception au rouge réservé à la destruction, et elle reste une teinte, jamais un
  aplat `st-c`.
- Avatars : `bg-av1` à `bg-av3` avec `text-av-tx`, choisis par `Avatar`.

### Correspondance des jetons v1

| v1 (hérité) | v2 |
| --- | --- |
| `bg-page` | `bg-app` |
| `bg-surface` | `bg-panel` |
| `bg-surface-alt` | `bg-group` |
| `bg-surface-elevated` / `bg-fill` | `bg-elev` / `bg-chip` |
| `border-line` / `border-line-soft` | `border-bd` / `border-bd-soft` |
| `border-control`, `border-field` | `border-bd-menu` |
| `text-ink` / `text-ink-muted` / `text-ink-faint` | `text-tx` / `text-tx-3` / `text-tx-5` |
| `text-accent`, `text-accent-text` / `bg-accent` | `text-ac-tx` / `bg-ac` |
| `bg-accent-tint` | `bg-tint-ac-bg` |
| `text-success` / `text-warning` / `text-danger` | `st-g` / `st-a` / `st-c` (`StatusGlyph`, teintes) |

---

## 4. Typographie

Interface en `font-sans` (system-ui), 12,5 px par défaut. Titres et scores en
`serif-title` (IBM Plex Serif 600, interlettrage resserré). Données en `font-mono` (IBM Plex
Mono). Nombres qui se rafraîchissent : `tabular`.

| Utilitaire | Taille | Famille | Où |
| --- | --- | --- | --- |
| `text-hero` · `text-score` · `text-stat` · `text-doc-score` | 32 · 26 · 25 · 22 px | serif | Scores, chiffres d’analyse |
| `text-screen` | 21 px | serif | Titre d’écran, nom d’une fiche |
| `text-overlay-title` · `text-empty` | 20 · 19 px | serif | Surcouche plein écran, état vide |
| `text-form` · `text-dialog` · `text-lead` | 17 · 16,5 · 15 px | serif | Formulaire modal, dialogue, mois du calendrier |
| `text-entry` | 13,5 px | sans 500 | Titre de carte, d’entrée |
| `text-row` | 13 px | sans | Ligne de liste, corps de lecture |
| `text-ui` | 12,5 px | sans | Défaut de l’interface, bouton |
| `text-small` · `text-sub` · `text-tiny` | 12 · 11,5 · 11 px | sans | Méta, aide, note |
| `caps` | 10,5 px | sans 500, capitales, `.04em`, `tx-6` | En-tête de section, de colonne |
| `text-caps` + `font-mono` | 10,5 px | mono | Référence, date courte, compteur |
| `text-kbd` | 9,5 px | mono | Touche (`Kbd`) |

---

## 5. Densité, dimensions, rayons

Candilog est **dense**. Fenêtre utilisable dès **940 × 560 px** ; la navigation se réduit
sous **1060 px** (préfixe `wide:`).

| Élément | Utilitaire | Valeur |
| --- | --- | --- |
| Barre de titre | `h-titlebar` | 64 px |
| En-tête de surcouche | `h-overlay-head` | 64 px |
| Barre d’outils d’écran | `h-toolbar` | 38 px |
| Barre d’état | `h-statusbar` | 34 px |
| Navigation | `w-nav` / `w-nav-sm` | 202 px / 52 px |
| Contrôle | `h-control` | 30 px |
| Ligne de candidature · relation · document | `h-row-app` · `h-row-rel` · `h-row-doc` | 34 · 38 · 40 px (30 · 34 · 36 en densité compacte) |

Les hauteurs de ligne suivent la préférence de densité (`data-density`, Réglages →
Apparence) : ne jamais les coder en pixels dans un composant.

| Rayon | Usage |
| --- | --- |
| `rounded-r2` · `r3` | Barre de graphique, jauge · case à cocher, feuille A4 |
| `rounded-r4` · `r5` | Touche · badge, pastille |
| `rounded-r6` · `r7` | Chip · contrôle (bouton, champ) |
| `rounded-r8` · `r9` | Carte · bloc, section |
| `rounded-r10` · `r11` | Carte de modèle IA · dialogue, formulaire modal |

Mouvement : couleur au survol en `duration-hover` (120 ms) ; entrée des menus et
dialogues `animate-pop` ; squelette `animate-sk`. La réduction de mouvement ramène toutes
les durées à 0,01 ms.

Focus clavier : `outline 1px accent-focus`, offset 0 — déjà sur `:focus-visible` global.

---

## 6. Icônes

Aucune police d’icônes (décision D7). Trois moyens, dans cet ordre :

- **`LineIcon`** (`src/shared/ui/LineIcon.tsx`) : les 12 tracés dessinés pour Candilog
  (navigation, signet, export, import, recherche, éprouvette du badge bêta) — grille 16,
  trait 1,4 px, `currentColor`. N’en ajouter un que s’il respecte cette grille — 16 × 16,
  trait 1,4 px, `currentColor`, aucun remplissage — et que le libellé seul ne suffit pas. Le
  dossier de tracés du handoff n’est plus dans le dépôt : `LineIcon.tsx` est désormais la
  définition de référence, et un tracé neuf s’y ajoute à côté des douze existants.
- **Glyphes typographiques** des maquettes : `‹ ›` (précédent, suivant), `▾` (liste,
  dépliable), `✕` (fermer, retirer), `✓` (choisi, terminé), `+`, `→`, `↶ ↷` ; en bouton
  seul, `GlyphButton`, qui exige un `label`.
- **`StatusGlyph`** quand l’icône portait un état (attention, erreur, réussite).

Sinon, pas d’icône : le libellé suffit. Une icône purement décorative à côté d’un titre ou
d’une ligne n’est pas remplacée. Marque : `BrandMark` (tuile `#5B62F0` fixe dans les deux
thèmes).

### Badge bêta

`BetaBadge` (`src/shared/ui/BetaBadge.tsx`) marque **tout ce qui passe par l’IA** : éprouvette
et mot « bêta », pastille ambre — ni une erreur, ni un état normal. Deux tailles : `nav` dans
une barre de navigation, la taille courante partout ailleurs. Son `title` donne la conséquence
(« relisez toujours ce que l’IA produit »), pas une définition de « bêta ».

Il paraît là où l’IA travaille, et nulle part ailleurs :

- barre de navigation et fil d’Ariane, sur la destination **Intelligence artificielle** — dans
  la barre il prend la place du décompte, et le libellé se tronque en conséquence (le `title`
  du lien le rend en entier). Sous le palier de 1060 px il disparaît avec les libellés : c’est
  une **enveloppe** qui porte `hidden wide:block`, jamais une classe posée sur le badge, dont
  l’`inline-flex` entrerait en concurrence avec elle et ferait déborder « bêta » du rail de
  52 px ;
- navigation de la surcouche Réglages, sur la même section ;
- en-tête (`badge` de `WorkSurface`) des générateurs de CV et de lettre, de l’analyse face à
  l’offre, de l’import de CV et de l’installation de l’IA locale ;
- pied de la colonne « Fournisseurs » de l’écran IA, où une phrase en donne la conséquence.

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
│ Barre de titre 64 px : fil d'Ariane · onglets de vue · mention│
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
  Sous les destinations, la section « Vues » (`SavedViewsNav`, masquée sous 1060 px) liste
  les vues enregistrées de Candidatures avec leur décompte ; `⋯` ou clic droit : renommer,
  dupliquer, supprimer (confirmé).
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
  Note ; Écrire, Relancer, Note), champs, candidatures rattachées, **Historique** (faits
  enregistrés lus par le backend : candidatures, statuts, entretiens, relances faites, notes,
  ajout de la fiche ; date `JJ-MM` en mono, candidature concernée sous le fait), notes de la
  fiche. « Note » et le `+` de l'historique ouvrent « Ajouter une note » (texte et date du
  fait) ; une note se supprime par son `✕`, après confirmation. `mailto:` et `tel:` restent
  des liens natifs ; un lien web passe par `openExternal`.
- « CSV » (`states/dialog-csv-rel.png`) : dialogue d'information qui annonce les deux
  fichiers (`entreprises-<date>.csv`, `contacts-<date>.csv`), leurs lignes et 9 colonnes,
  le séparateur et l'encodage ; puis le dialogue natif. Tout est exporté, sans la recherche
  (`DECISIONS.md` E10) ; le fichier des contacts est écrit à côté de celui des entreprises.

### Documents

- Un seul écran (`DocumentsPage`) : onglets Tous / CV / Lettres / Analyses dans la barre
  d'outils (routes `/documents`, `/documents/resumes`, `/documents/letters`,
  `/documents/analyses`), recherche `/`, actions Importer, Générer une lettre, CV de base,
  Générer un CV (`N`).
- Liste groupée CV / Lettres de motivation, lignes de 40 px : feuille, nom et sous-titre,
  score ATS en pastille (vert dès 80, accent dès 65, rouge en dessous), date, entreprise.
  Clic droit : Dupliquer ou Copier le texte, Supprimer.
- Inspecteur 300 px (dès 1060 px) : Ouvrir (éditeur A4, qui porte l'aperçu pleine page),
  PDF (`⌘E`), score ATS et constats, détails, **Versions** (`v3`, mention, date ; la
  courante est marquée). Cliquer une autre version ouvre « Revenir à la version vN ? »
  (registre confirmation : version courante, version restaurée, versions conservées) ;
  rien n'est effacé. Enregistrer un document rouvert depuis l'inspecteur en ajoute une
  version. La comparaison de versions est hors v2. Une ancienne version sans contenu
  structuré le dit au lieu d'offrir un export vide.

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

- Surcouche plein écran (`WorkSurface` dans `shared/ui`, `GeneratorFrame` pour Documents) :
  `×` ou `Échap` ferme (sauf pendant une
  génération), fil « Documents › Générer … », actions à droite, trois colonnes — réglages
  (260 px), feuille A4 sur le bureau `bg-canvas`, déroulé (270 px, dès 1060 px) — et barre
  d'état. `⌘⏎` génère, `⌘S` enregistre : la surcouche porte ses propres raccourcis, ceux
  d'écran se taisant sous une surface.
- **Profil à compléter** (`ProfileGapBanner`, bandeau ambre en tête de la colonne de gauche)
  dès que le profil n'a pas de quoi nourrir la génération : pas de nom, ou aucune expérience,
  formation ni compétence (`model/profileReadiness.ts`). Un générateur n'invente pas un
  parcours, il reformule le profil : le dire avant vaut mieux que de laisser découvrir une
  feuille creuse. Il n'empêche pas de générer, et « Compléter le profil » quitte la surcouche
  — rien n'est encore produit à cet instant.
- **CV de base** (`BaseResumePage`, route `/documents/base-resume`) : même `GeneratorFrame`,
  mais sans colonne droite — rien à attendre, aucune IA n'intervient — et sans `BetaBadge`,
  qui signale précisément l'inverse. La feuille se compose dès l'ouverture, depuis le seul
  profil ; à gauche, `SectionToggles` sur sa propre liste (`BASE_RESUME_SECTIONS`) : sept
  interrupteurs, « Présentation » en tête — elle ouvre le CV — puis expériences, formations,
  compétences, langues, projets et certifications. Ni disponibilité ni centres d'intérêt :
  aucun champ du document ne les accueille, et un réglage sans effet serait un mensonge.
  Viennent ensuite `ProfileGapBanner` et le nom de la version. `⌘S` enregistre, `Échap` ferme ; pas de `⌘⏎`, il n'y a rien à générer.
  Basculer un interrupteur recompose depuis le profil et écrase les retouches ; une
  confirmation le demande dès que la feuille a été retouchée. Quand la composition est
  refusée et qu'aucune feuille n'est affichée — profil vide, ou profil sans nom, qu'un CV
  ne peut pas porter — le centre remplace le squelette A4 par un seul état vide portant le
  message du natif et « Compléter le profil ». `ProfileGapBanner` se tait alors : il annonce
  que rien n'est bloqué, et il le contredirait. Il reparaît dès qu'une feuille existe, où il
  signale ce qui manque sans empêcher d'enregistrer. Rouvert depuis la
  bibliothèque, un CV de base affiche ce qui a été enregistré, sans recomposer, avec ses
  interrupteurs dans l'état où la composition les avait laissés, et un nouvel
  enregistrement y ajoute une version. Dans la bibliothèque, il ne se distingue des
  autres CV en rien — pas de pastille, pas de filtre — décision produit, pas un oubli.
- « Offre visée » (`OfferSource`) : s'ouvre sur les candidatures ouvertes du suivi — le
  texte est prérempli avec ce que Candilog en sait, sans rien inventer — sauf si une offre
  est déjà fournie ; sinon le texte collé.
- « Étapes », puis « En cours » et « Terminé » : les étapes annoncées par le backend, dans
  l'ordre prévu, avec la durée **mesurée** de chacune en secondes entières (`useStepLog`) ;
  une étape dont l'annonce a été manquée mais qui est dépassée est faite, sans durée. Sous
  les étapes pendant le traitement, `RunMeter` : barre des étapes terminées, temps écoulé,
  tokens rapportés. La barre d'état dit « génération en cours · étape 2 / 4 ».
- « Arrêter » (`⌘.`) ouvre « Interrompre la génération ? » (`StopGenerationDialog`) :
  l'étape en cours, le temps écoulé, « Laisser finir » ou « Interrompre ».
- « Ce que l'IA peut utiliser » (CV) et « Arguments autorisés » (lettre) — `SectionToggles` :
  une ligne par section du profil avec son nombre d'éléments et un interrupteur ; une section
  vide ne se règle pas ; l'en-tête compte les sections autorisées (« 5 / 6 »). Le CV a un
  ton (Sobre, Professionnel, Direct) mais pas de longueur : il tient sur une page. Aucun
  réglage que le backend ne sait pas honorer n'est affiché (pas de prétentions salariales).
- Analyse de CV (`screens/09`) : même surcouche sans colonne droite. À gauche, le CV (PDF)
  « face à » l'offre, puis le score calculé par Candilog et son détail ; au centre, chaque
  exigence de l'offre (`score.evaluations`) — couverte, partielle, absente — avec la preuve
  citée du CV ou « introuvable ». Une exigence manquante sans évaluation détaillée reste
  listée comme absente. Le formulaire est figé, pas masqué, pendant l'analyse.
- Import de CV (`screens/17`) : même surcouche rattachée au Profil — méthode d'analyse à
  gauche, phases au centre (choix du fichier, analyse, revue élément par élément éditable,
  bilan), « Importer les éléments sélectionnés » (`⌘S`) en haut. « Annuler » arrête aussi
  une analyse en cours ; rien n'est écrit avant validation.
- Éditeur de CV, panneau ATS (`ResumeAtsPanel`) : le score porte un « jusqu'à N » et la
  barre une seconde teinte dès qu'il reste des actions — jumeau du `LetterFitPanel`. Le
  document s'ouvre sans compétences (elles attendent dans la bibliothèque du profil), donc un
  profil bien adapté affiche un score bas à la génération : sans ce repère, ce chiffre se lit
  comme un verdict au lieu d'un point de départ. C'est la **somme des gains affichés**,
  plafonnée à 100, pour qu'elle tombe juste par rapport à la liste sous les yeux — un plafond
  indicatif, d'où « jusqu'à », et non une promesse.
- Lettre, colonne droite après rédaction (`LetterFitPanel`) : « 62 / 100 · adéquation »,
  « jusqu'à 81 » (score si toutes les recommandations restantes étaient suivies), barre à
  deux teintes ; recommandations « Aborder « … » » avec le fait du profil cité, gain `+n`,
  « Appliquer » (consigne envoyée aux corrections) et « Ignorer » ; puis « Absent de votre
  profil ». Sans offre, le panneau n'apparaît pas.
- Lettre : avant rédaction, feuille neutre et lien « Écrire la lettre moi-même » qui ouvre
  l'éditeur ; entreprise, poste, destinataire, ton et longueur restent visibles ; la barre
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
- **« Grouper : statut ▾ »** (barre de titre, Liste seulement) : cycle en place statut →
  entreprise → contrat, aussi dans `⌘K` (« Affichage »). Hors statut, les groupes et leurs
  décomptes viennent de SQLite (`applications_groups`, tout le filtre) ; les huit premiers
  sont ouverts, un groupe replié n'est pas chargé. Une action de barre de titre est une
  commande nommée (`useChrome({ action })`), le chrome étant sérialisé.
- Recherche : `/` y place le focus, `Échap` l'efface puis la quitte.
- **Vues enregistrées** : « Enregistrer la vue » nomme le filtre courant ; une vue ouverte
  (`?view=<id>`, `ApplicationsRoute`) donne son nom au fil d'Ariane et propose « Mettre à
  jour la vue » dès que le filtre s'en écarte. Le lien entre Candidatures et vues vit dans
  la couche `app`, jamais d'une feature à l'autre.
- **Barre groupée** (`BulkBar`) : changer le statut (`S`), exporter en CSV (`⌘E`),
  supprimer (`⌘⌫`), désélectionner (`Échap`). Elle agit sur les cases cochées, jamais sur
  la seule fiche ouverte.
- **Kanban** : colonnes élastiques 236–340 px sur `bg-group`, défilement horizontal ;
  bandeaux « sans réponse depuis plus de 14 jours » et « entretien aujourd'hui » ; cartes
  compactes (référence, échéance, intitulé, entreprise, contrat, date) ; chaque colonne est
  paginée côté SQLite (`ColumnPager`). Un changement de statut affiche `CAN-142 → Entretien`.
  Le déplacement d'une carte utilise le glisser-déposer **HTML5** (`dataTransfer`), ce qui
  impose `dragDropEnabled: false` dans `src-tauri/tauri.conf.json` : laissé à `true`, le
  gestionnaire de dépôt natif de Tauri intercepte les événements et le glisser-déposer HTML5
  ne fonctionne plus sous WebView2 (Windows). Rien n'écoute `tauri://drag-drop`, donc le
  drapeau ne rendait aucun service et ouvrait une surface de dépôt de fichiers inutilisée.
  Ne pas le remettre à `true` sans câbler le dépôt natif **et** vérifier le Kanban sous
  Windows : les deux mécanismes sont mutuellement exclusifs sur cette plateforme.

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

- Aucune bibliothèque de graphiques (décision D7) : des primitives du design dans
  `analytics/view/components/charts` — barres en blocs aux jetons de couleur (`bg-ac`,
  `bg-chip` pour un zéro), repère de la valeur maximale, étiquettes espacées sur une série
  longue. Les jetons suivent le thème sans re-render.
- Aucune valeur accessible par le seul survol : axe visible ou liste `sr-only` équivalente.
- Une seule série → pas de légende, la carte la nomme. Deux séries ou plus → légende
  systématique, avec libellé **et** compte, car les teintes de statut vert et rouge sont
  proches pour une deutéranopie.
- États vides gérés par le graphique lui-même (`EmptyState`), pas par l’écran appelant.

### Réglages (surcouche `⌘,`)

- Surcouche plein écran (`screens/18-settings.png`) : `✕` et fil « Réglages › section » dans
  la barre de titre, version à droite, sections à gauche, barre d'état « appliqué
  immédiatement · propre à cet ordinateur ».
- Chaque section : `SettingsSection` (titre serif, phrase qui dit l'**effet**) puis des
  `SettingsRow` — libellé et conséquence à gauche, contrôle à droite, filet bas. Apparence,
  Données (sauvegarder, restaurer, réinitialiser — chacune confirmée), Raccourcis, Mises à
  jour (état, version installée et nouvelle, action, progression, nouveautés), À propos.
- Rien sur le mécanisme de téléchargement d'une mise à jour : cela n'aide pas à décider.
- Section Intelligence artificielle : renvoi vers l'écran IA et ligne « Confirmation avant
  envoi distant » (`RemoteSendConsents`) — les services dispensés, « Redemander ».
- Premier envoi distant (`RemoteSendDialog`, `states/dialog-confirm-remote-send.png`) :
  registre confirmation, « Cette tâche sera envoyée à Anthropic », ce qui part et pour quelle
  tâche, conséquences (destinataire, tâche, « 1 sur 5 »), interrupteur « Ne plus demander
  pour … » **désactivé** par défaut, « Annuler » / « Envoyer ». Monté une fois dans la coque.

- IA (`screens/12`, `13`) : colonne **Fournisseurs** de 190 px (onglets verticaux : état « Local · n modèles », « Clé enregistrée », « Aucune clé », point vert quand le fournisseur est prêt, mention « principal » ; en pied de colonne, le `BetaBadge` et la phrase qui en donne la conséquence — relire tout ce que l'IA produit), puis le détail : nom en serif, description **factuelle** et trois jauges — confidentialité, coût, hors connexion ; jamais une promesse de qualité. L'IA locale affiche ses modèles installés en lignes (`ManagedOllamaPanel` : Utiliser, Tester, Supprimer) et ouvre **Installer l'IA locale** (`LocalInstallOverlay`, `screens/14`, sur `WorkSurface`) : à gauche les trois étapes et la fiche du modèle choisi, au centre les modèles en boutons radio avec jauges, puis le déroulé et « Installation terminée » ; annulation et échec s'affichent en place, jamais par toast, un fournisseur distant sa configuration (modèle et `RemoteModelPicker` après Actualiser, clé jamais rendue en clair, mode, température). **L'endpoint n'apparaît que là où il décrit un choix** : « Personnalisé », dont c'est l'objet, et Ollama, dont l'utilisateur désigne l'hôte — l'adresse des cinq services cloud est fixe, et l'exposer n'offrait aucun réglage tout en laissant une faute de frappe casser le fournisseur en silence. Une valeur déjà personnalisée reste visible, pour rester corrigeable avec « Tester la connexion » (`T`) et « Enregistrer », qui en fait le fournisseur principal. En bas, **Qui fait quoi** (`AiTaskRouting`, routage décrit dans `docs/AI.md`) : cinq tâches, le modèle de chacune, point vert (local), ambre (distant) ou gris (désactivée) ; le sélecteur propose le fournisseur principal, les modèles installés, les fournisseurs distants configurés et « Aucun » ; le choix est enregistré aussitôt, sans repli. Le fournisseur local s’appelle **« IA locale »**, jamais d’après une famille de modèles. Même principe pour les distants : la carte porte le nom de l’**éditeur** — Anthropic, Google, Mistral AI, OpenAI, DeepSeek — et la famille de modèles passe en indice. C’est l’éditeur qui reçoit les données, donc lui que l’utilisateur doit reconnaître avant d’accepter un envoi ; `taskRouting` réutilise ce libellé comme destinataire plutôt que d’en tenir une seconde liste. La liste **Profils disponibles** reprend les `evaluations` du backend (`compatibility` + `reason`) : un profil `unsupported` est étiqueté « Incompatible » et son installation est désactivée. Un benchmark `too_slow` avertit **toujours**, y compris sur le plus petit profil où aucun repli n’existe. Le résultat d'un test de connexion est un message fixe : la prose du modèle n’est pas un état.

### Formulaires

- Toujours `ModalHost` (620 px par défaut, `rounded-r11`, pied fixe visible).
- Champs : `FormField` + `TextInput` / `Select` / `TextArea` / `DateInput` / `TimeInput` / `EntityPicker`.
- Erreur **sous** le champ (`aria-invalid`, `aria-describedby`), jamais une infobulle seule.
- Requis : mention `obligatoire` en ambre (`st-a`) tant que le champ est vide, pas de contour rouge.

### Destruction

- `ConfirmDialog` (440 px). Titre en question. Description : ce qui disparaît. `note` : ce qui survit.
- Confirm = `danger`. Pas de toast à la place d’une confirmation.

### Feedback

| Situation | Composant |
| --- | --- |
| Chargement d’écran | `Skeleton` / `SkeletonRows`, `role="status"` |
| Échec de chargement | `ErrorBanner` + Réessayer |
| Avertissement non bloquant (ce qui est affiché reste utilisable, le résultat vaudra ce que vaut ce qui manque) | `Banner` `tone="warning"` — ambre, glyphe `a` |
| Succès / échec d’écriture sans décision | `notify()` → `Toaster` (2,6 s, centré au-dessus de la barre d’état) |
| Nouvelle version trouvée au démarrage | `notify()` `info` — titre factuel, destination en détail, **jamais** de bouton (`docs/RELEASES.md`) |
| Décision destructive | `ConfirmDialog` |
| Rien à montrer | `EmptyState` dans le contenant, pas un écran plein décoratif |
| Traitement IA en cours | `AiProgress` : étape, barre indéterminée et temps écoulé — **jamais** de pourcentage, il serait inventé |
| Traitement IA terminé | Durée totale en badge d'en-tête (« Rédigée en 12 s ») |
| Fonction qui passe par l'IA | `BetaBadge` dans l'en-tête de l'écran ou de la surcouche (§6) |

---

## 10. Composants — quand les prendre

Toujours importer depuis `@/shared/ui`.

| Besoin | Composant |
| --- | --- |
| Action | `Button` (`primary` \| `secondary` \| `ghost` \| `danger` \| `link`), raccourci par `shortcut` |
| Touche imprimée | `Kbd` (`⌘K` sous macOS, `Ctrl K` ailleurs) |
| Statut | `StatusGlyph` (`n` · `a` · `g` · `c`) ; pastille sans statut : `Tag` |
| Entreprise, personne | `Avatar` |
| Interrupteur | `Switch` |
| Choix court (2 à 4 options) | `SegmentedControl` |
| Menu (clic droit, `⋯`, « + Filtre ») | `Menu` (entrées, sections, second niveau avec `onBack`) |
| Confirmation | `ConfirmDialog` (registres destruction, confirmation, information) |
| Formulaire | `ModalHost` + `FormField` / `TextInput` / `Select` / `DateInput` / `EntityPicker` |
| Surcouche plein écran | `WorkSurface` + `PaneSection` / `StepList` / `RunMeter` |
| Split redimensionnable | `SplitPane` |
| Pagination | `Pager` / `ColumnPager` |
| Chargement, vide, erreur | `Skeleton`, `EmptyState`, `ErrorBanner` |
| Icônes | `LineIcon`, glyphe typographique ou `GlyphButton` (§6) |
| Graphique | `analytics/view/components/charts` — primitives du design, liste `sr-only` des valeurs |

La planche `/_design` (`src/app/dev/DesignGallery.tsx`, développement seulement) montre ces
primitives dans les deux thèmes.

`Card` existe pour des blocs denses déjà dans le design ; ne pas s’en servir pour recréer un dashboard de widgets.

---

## 11. Overlays

**La coque n'est pas vitreuse.** `--candilog-blur-shell` et `--candilog-blur-overlay` valent
`0px` : la hiérarchie vient des surfaces et du filet 1 px (§3), conformément à §2. Les classes
`glass-topbar`, `glass-subnav`, `glass-inspector`, `glass-menu` et `glass-modal` de la v1 ont
été retirées de `styles.css` — elles n'avaient plus aucun appelant. Seule `glass-popover`
subsiste, employée par `DateInput`, et elle ne pose qu'une couleur de fond.

Un overlay (popover Filtres adaptatif de 230 à 640 px selon son contenu, menu, dialogue) se
construit donc avec les jetons de surface — `bg-menu`, `bg-modal` — un filet `border-bd-menu`
et l'ombre unique `shadow-pop` de §2.

Fermeture : `useDismissable` (Escape + clic extérieur) — calendrier, FilterMenu, inspecteur,
modale.


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

- Chaque écran est nommé : fil d’Ariane de la barre de titre, `h1` en serif pour une fiche,
  une surcouche ou un générateur.
- Focus visible global. Pas de `outline-none` sans remplacement.
- Bouton Filtres : `aria-label` « Filtres » ou « Filtres, n actifs ».
- Chips : `aria-label` « Retirer le filtre … ».
- Empty / loading : `role="status"` ou `alert` selon le cas.
- Contraste : pas de `bg-ac` sous un long texte ; texte accentué en `text-ac-tx`.
- `prefers-reduced-motion` respecté (palette).

---

## 13. Checklist agent

Avant de merger un changement d’UI :

1. J’ai réutilisé un composant de `shared/ui` plutôt que d’en créer un visuellement proche.
2. Aucun hex / `rgb()` nouveau dans le TSX.
3. Contrôles à 30 px, filets 1 px, pas d’ombre sur le contenu, jetons v2 seulement.
4. Libellés français, identifiants anglais.
5. Liste paginée : filtre et recherche côté backend, barre d’outils d’écran (`Toolbar`, « + Filtre »).
6. Pas de pile technique ni de hero marketing.
7. États vide, erreur, chargement traités.
8. Vérifié clair **et** sombre (classes sémantiques, pas de `bg-white`).

En cas de doute, copier **Candidatures** (liste groupée), **Relations** (maître-détail) ou une section des **Réglages** — pas un template externe.
