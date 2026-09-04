# Design system — candilog.fr

Référence des valeurs du site. Ce document remplace le dossier de handoff
`design_handoff_candilog_landing/` (prototypes `.dc.html` et README d'intégration),
qui n'a pas vocation à vivre dans le dépôt : tout ce qu'il fallait en retenir est ici.

Le design est **haute fidélité** : les valeurs ci-dessous sont des décisions, pas des
approximations. `19px` n'est pas `h-5`, `13,5px` n'est pas `text-sm`.

---

## 1. Le principe

Les couleurs ne sont pas écrites dans les classes, elles sont **exposées à Tailwind
depuis des variables CSS** (`app/globals.css`, bloc `@theme inline`). `bg-surface` rend
blanc en clair et `#0f1116` en sombre, sans une seule variante `dark:`.

```tsx
// ✅ une seule écriture, les deux thèmes
<div className="rounded-card border border-line bg-surface text-ink">

// ❌ double maintenance, et ce n'est pas la palette
<div className="rounded-xl border border-gray-200 bg-white dark:bg-gray-900">
```

### Les cinq règles

1. **Aucune couleur Tailwind par défaut.** Pas de `bg-white`, `text-gray-500`,
   `bg-indigo-600`. Si une couleur manque, c'est qu'elle n'existe pas dans le design.
2. **Aucune variante `dark:`.** Elle entre en conflit avec les tokens et casse le thème.
3. **Les valeurs hifi passent par les crochets** : `h-[19px]`, `text-[13.5px]`,
   `px-[7px]`, `gap-[9px]`.
4. **`style={{}}` uniquement pour le dynamique** — largeur calculée, rotation d'état,
   dégradé de marque. Jamais une couleur ou un espacement fixe.
5. **Les variantes dans un objet**, pas dans des ternaires imbriqués. Voir `STATUT`
   (`lib/data/suivi.ts`) et `VARIANTES` (`components/ui/Button.tsx`).

---

## 2. Tokens

Définis en clair sur `:root` et en sombre sur `:root[data-theme="dark"]`.

| Famille | Utilitaires |
| --- | --- |
| Surfaces | `bg-page` `bg-surface` `bg-surface-alt` `bg-surface-sunken` `bg-overlay` |
| Filets | `border-line` `border-line-soft` `border-control` `border-control-strong` |
| Texte | `text-ink` `text-ink-body` `text-ink-muted` `text-ink-tertiary` `text-ink-faint` |
| Accent | `bg-accent` `bg-accent-strong` `text-accent-text` `text-on-accent` |
| Teintes | `bg-tint-03` → `bg-tint-12`, `border-tint-border`, `border-tint-border-strong` |
| Sémantique | `{bg,text,border}-{success,warning,danger}[-tint\|-text\|-border]` |
| Bande ATS | `bg-band` `bg-band-surface` `bg-band-alt` `bg-band-elevated` `text-band-ink…` |

**La bande CV/ATS a son propre jeu `--band-*`** : claire en thème clair, légèrement plus
profonde que la page en thème sombre. Ne pas la repeindre avec les tokens de page — son
contraste est différent volontairement.

| Rayons | Ombres | Courbes |
| --- | --- | --- |
| `rounded-pill` 6px | `shadow-menu` | `ease-out-soft` — `cubic-bezier(.2,.7,.2,1)` |
| `rounded-control` 8px | `shadow-tile` | `ease-reveal` — `cubic-bezier(.16,1,.3,1)` |
| `rounded-tile` 10px | `shadow-tile-hover` | |
| `rounded-card` 12px | | |
| `rounded-panel` 14px | | |
| `rounded-app` 22px | | |

**Aucune ombre sur le contenu.** Elles sont réservées aux overlays (menus) et aux
pastilles IA.

---

## 3. Thème clair / sombre

- Piloté par `data-theme="light" | "dark"` sur `<html>`, mémorisé sous la clé
  `candilog-theme`, avec `prefers-color-scheme` en repli au premier chargement.
- **Le script anti-flash de `app/layout.tsx` est obligatoire** : il pose l'attribut avant
  le premier paint. Sans lui, un visiteur en mode sombre voit un flash blanc.
- `ThemeToggle` : bouton carré de 30px, dernier élément des actions de l'en-tête, icône
  `dark_mode` ↔ `light_mode` avec rotation de 180° sur 320 ms.
- Les logos de marque monochromes passent par `BrandIcon`, qui utilise le SVG en
  `mask-image` avec `bg-current` : le logo prend le token de texte de son conteneur et
  suit le thème seul. C'est ce que le prototype encodait en dur dans ses URL
  `cdn.simpleicons.org/<marque>/<couleur>`.

---

## 4. Typographie et icônes

| Usage | Valeur |
| --- | --- |
| Texte courant | `system-ui, -apple-system, "Segoe UI", sans-serif` |
| Métadonnées, chiffres, codes | **JetBrains Mono** 400/500/600, via `next/font` |
| Icônes | **Material Symbols Rounded**, `opsz 20..48, wght 300, FILL 0, GRAD 0` |

Les deux polices sont auto-hébergées. Le composant `Icon` fixe les
`font-variation-settings` ; ne pas écrire de `<span className="material-symbols-rounded">`
à la main.

**Échelle de tailles réellement utilisée (px)** : 9,5 · 10 · 10,5 · 11 · 11,5 · 12 ·
12,5 · 13 · 13,5 · 14 · 14,5 · 15 · 15,5 · 16 · 17 · 19 · 20 · 36, puis les titres en
`clamp()`.

| Titre | Valeur |
| --- | --- |
| H1 landing | `clamp(30px, 3.6vw, 44px)` · 600 · `-0.022em` · `1.1` |
| H2 sections | `clamp(24px, 2.6vw, 34px)` · 600 · `-0.02em` · `1.14` |
| H2 bande ATS | `clamp(24px, 2.8vw, 36px)` |
| H2 téléchargement | `clamp(25px, 2.8vw, 38px)` |
| H1 pages légales | `clamp(28px, 3.2vw, 40px)` · `1.12` |
| H2 pages légales | 20px · `-0.014em` · `1.3` |

---

## 5. Layout

- Contenu : **1240px** centré, gouttières `clamp(16px, 4vw, 40px)`.
- Pages légales : colonne unique de **800px** centrée.
- Hauteurs de contrôles : **30px** compact · **34–36px** secondaire · **38–40px**
  principal. Jamais moins de 30px.
- Filets : `border-line` en séparateur de section, `border-line-soft` dans les cartes.
- Transitions : **120 ms** boutons et liens · **160–220 ms** états · **320 ms**
  ouvertures · **640/720 ms** apparitions au scroll.
- Groupes de frères : `flex` + `gap`. Jamais de marges individuelles.

---

## 6. Responsive — comportements à ne pas perdre

Vérifié à 360 / 390 / 430 / 768 / 1024 / 1440 px sur les 5 pages : aucun débordement
horizontal. **Les `overflow-x` ci-dessous sont structurels, pas décoratifs.**

| Bloc | Comportement |
| --- | --- |
| En-tête | Deux à trois lignes sous ~700px, une seule ligne de 56px dès la tablette. Nav en `overflow-x-auto` + `no-scrollbar`. |
| Fenêtre du hero | `min-w-[760px]` sur la zone de contenu, `overflow-hidden` sur la fenêtre : elle se **coupe** au bord droit au lieu d'exposer une barre de défilement dans une maquette décorative. Déborde à droite au-delà de 1180px via `mr-[calc(-1*clamp(0px,(100vw-1180px)*0.35,110px))]`. |
| Frise des 5 étapes | `grid-flow-col` + `overflow-x-auto`, colonnes de 158px minimum. |
| Kanban de suivi | `repeat(auto-fit,minmax(min(240px,100%),1fr))`, 4 colonnes — la géométrie de `KanbanBoard`. |
| Vue liste | `overflow-x-auto` + `min-w-[940px]` : ses **huit** colonnes `2.2fr 1.3fr .9fr .7fr 1fr .8fr 1.1fr .8fr` — celles de `ApplicationsPage` — deviennent illisibles en dessous. |
| Grille des 4 faits | `repeat(2, minmax(0,1fr))` **fixe**. Ne pas repasser en `auto-fit` : les filets se désalignent. |
| Pastilles IA | `flex-wrap`, **sept** pastilles. Décalages alternés 26/4 : à sept, la ligne se replie selon la largeur et une vague irrégulière ferait des marches au point de repli. |

> ⚠️ Le hero porte `overflow-x-clip`, et c'est **`clip`, pas `hidden`**. Sans clip, le
> débordement volontaire de la fenêtre crée une barre horizontale sur toute la page. Et
> `overflow-x: hidden` forcerait `overflow-y` à `auto`, transformant la section en
> conteneur de défilement — une barre verticale de 15px apparaît alors dans le hero.

---

## 7. Animations

Quatre blocs. Les trois pilotés par JavaScript sont **tous** conditionnés à
`prefers-reduced-motion: reduce`, et aucun n'est une animation CSS infinie : ils s'arrêtent
et se nettoient au démontage.

| Où | Comportement |
| --- | --- |
| `useScrollReveal` | Montée de 16px + fondu, 80 ms entre enfants, seuil 0.08. Appliqué via `<Reveal>` sur le conteneur de chaque section. |
| `useAtsReveal` | Score ATS de 0 à **84**, +3 toutes les **26 ms**, à l'entrée dans le viewport (`IntersectionObserver`, seuil **0.35**), une seule fois. Le gain cumulé (**+13**) est posé d'emblée : l'application l'affiche, elle ne l'anime pas. |
| `animate-caret` | Caret du champ « Texte de l'offre » (écran 01 du parcours) : `steps(1)`, 1,1 s, infini. Seule animation purement CSS du site, sur un élément `aria-hidden` de 1px. |
| `TrackingBoard` | La carte « Designer produit » **change de colonne** toutes les **2 600 ms** : En attente → Relancée → Entretien, avec `animate-depose` (320 ms) à l'arrivée et le compteur d'en-tête qui suit. Dans l'application une carte ne porte pas son statut — c'est la colonne qui le porte, et on en change en glissant la carte : l'animation reproduit ce geste, elle ne repeint pas une pastille inexistante. |

Le rendu serveur sort **sans** les classes `.reveal` : sans JavaScript, tout reste
visible.

> ⚠️ `.reveal-in` **libère** le `will-change` posé par `.reveal`, et ce n'est pas une
> optimisation. Sous Chromium, `will-change: opacity | transform` fait de l'élément une
> *backdrop root* : tant que le drapeau tient, le `backdrop-filter` de ses descendants n'a
> plus rien à flouter. Le menu de téléchargement — seul verre dépoli posé à l'intérieur
> d'un `.reveal` — laissait alors lire le texte du hero au travers de sa première bande.
> Ne pas remettre `will-change` sur `.reveal-in`.

---

## 8. Accessibilité — acquis à conserver

- `:focus-visible { outline: 1px solid var(--accent); outline-offset: 2px }` global.
- Cibles ≥ 30px, et **≥ 44px sur mobile** pour les liens de nav (`min-h-[44px] md:min-h-0`).
- Le parcours est un vrai `role="tablist"` : `aria-selected`, roving `tabindex`,
  navigation aux flèches + `Home`/`End`.
- La bascule board/liste est un `role="group"` de deux boutons avec `aria-pressed`.
- FAQ et menus de téléchargement sont des *disclosures* : `aria-expanded` +
  `aria-controls`. **Leur panneau fermé porte `inert`** — sans ça, les liens qu'il
  contient restent atteignables au clavier alors qu'ils sont invisibles.
- Les deux `DownloadMenu` (hero et CTA) ne sont jamais ouverts en même temps
  (`lib/menuOuvert.ts`). Fermeture au clic extérieur et à `Échap`, focus rendu au
  déclencheur. Le conteneur `.reveal-in` qui possède un menu ouvert est élevé au-dessus des
  sections suivantes : son `transform` crée un contexte d’empilement que le `z-index` du
  panneau ne peut pas traverser seul.
- La fenêtre du hero est décorative : `aria-hidden`, zéro élément focusable.
- Icônes décoratives : `aria-hidden="true"` (posé par `Icon` et `BrandIcon`).

### Contrastes mesurés en thème sombre

| Paire | Ratio |
| --- | --- |
| `--ink` sur `--page` (titres) | 16,40 |
| `--ink-body` sur `--surface` (texte courant) | 11,88 |
| `--on-accent` sur `--accent` (boutons accent) | **3,53** |
| `--control-strong` sur `--surface` (numéros d'étape inactifs) | **1,80** |
| `--ink-faint` sur `--surface-alt` (sur-titres mono) | **3,73** |

Les trois dernières sont sous les seuils WCAG AA. Ce sont des **choix de palette**, pas
des accidents d'intégration : le prototype donnait exactement les mêmes valeurs. Les
corriger implique de toucher aux tokens.

---

## 9. Contenus juridiquement calibrés

> ⚠️ Les textes des 4 pages légales, et les réponses de FAQ portant sur la
> confidentialité, la licence et l'ATS, sont **calibrés**. Ils ne promettent ni
> « aucune donnée ne quitte votre ordinateur », ni un résultat de recrutement. Ne pas
> les reformuler sans relecture.

Mentions obligatoires à conserver telles quelles :

- « Une analyse est une indication, pas une garantie de sélection. » (bande CV/ATS)
- Le § « Site internet » de la politique de confidentialité affirme que le site ne fait
  aucune requête vers un service tiers. **Ajouter une ressource externe rend ce texte
  faux** — mettre le texte à jour, ou renoncer à la ressource.

---

## 10. Fidélité des aperçus

Les maquettes d'écran du site — fenêtre du hero, cinq écrans du parcours, Kanban et liste
de la section Suivi, trois panneaux de la bande ATS — ne sont pas des illustrations
libres. **Elles ne montrent que ce que l'application fait.** Un site qui met en scène une
fonctionnalité absente est une promesse cassée au premier lancement.

### La règle

`src/` fait foi. Avant de modifier un aperçu, lire le composant réel qu'il représente et
en reprendre la structure, les libellés, les colonnes et les états.

| Aperçu | Source de vérité dans `src/` |
| --- | --- |
| Fenêtre du hero | `app/layout/{AppShell,NavRail,TopBar,SubNav}.tsx`, `features/applications/view/pages/ApplicationsPage.tsx` |
| Statuts (`lib/data/suivi.ts`) | `features/applications/model/statuses.ts` |
| Kanban et cartes | `features/applications/view/components/{KanbanBoard,ApplicationCard}.tsx` |
| Vue liste | les colonnes de `ApplicationsPage.tsx` |
| Écrans 01 et 02 du parcours | `features/documents/view/pages/ResumeGeneratorPage.tsx`, `view/components/ResumeAtsPanel.tsx` |
| Écran 03 | `features/documents/view/pages/ResumeLibraryPage.tsx` |
| Écran 04 | `features/applications/view/components/ApplicationFormModal.tsx` |
| Écran 05 | `features/calendar/view/pages/CalendarPage.tsx`, `view/components/GridMonth.tsx` |
| Bande ATS | `features/documents/view/components/{ResumeAtsPanel,DocumentUi}.tsx` |
| Fournisseurs IA | `features/settings/model/providers.ts` |

### Ce que l'application ne fait pas

À ne pas remettre en scène, quelle que soit la qualité graphique du résultat :

- **Pas de section « Offres »**, pas d'import d'annonce par URL, pas d'extraction
  automatique du poste et de l'entreprise. Une offre entre par son **texte collé**, dans
  « Générer un CV » ou « Analyser ».
- **Pas de cinquième statut.** Il y en a quatre : En attente, Relancée, Entretien,
  Refusée. Aucun statut « offre reçue ».
- **Pas de pièces jointes** sur une candidature : les CV et lettres vivent dans la
  bibliothèque de documents.
- **Pas de jauge de lisibilité, pas de verdict global.** L'analyse ATS rend un score sur
  100, un gain cumulé et des propositions à décider une par une.
- **Pas de barre de titre dessinée** : `tauri.conf.json` garde les décorations natives,
  donc ni pastilles macOS ni boutons Windows dans la maquette.

### Le jeu de données

Un seul persona sur tout le site : **Camille Berthier**, designer produit à Lyon, avec
Atelier Nord, Studio Halage, Cobalt Bureau, Groupe Vallée, Sablé Industries, Éditions
Sillon, Nord Réseaux, Maison Rivet, Laurier & Pons et Verrières & Cie. Les dates suivent
le format de l'application (**JJ-MM-AAAA**), les entretiens et relances tombent des jours
ouvrés, et les compteurs sont cohérents entre les sections : **14 candidatures**,
réparties 5 / 3 / 3 / 3.

---

## 11. Écarts assumés par rapport au prototype

Le prototype ayant disparu du dépôt, voici ce qui a été fait différemment et pourquoi.
Sans cette liste, ces choix ressemblent à des erreurs.

| Écart | Raison |
| --- | --- |
| Icônes et polices auto-hébergées (`material-symbols` en npm, `next/font`) au lieu des CDN Google | Aucune dépendance réseau tierce ; cohérent avec l'app desktop. C'est ce qu'affirme la politique de confidentialité. |
| Logos de marque en `mask-image` + `currentColor` au lieu de `filter: invert(1) hue-rotate(180deg)` | Le logo suit le token de texte, donc le thème, sans variante conditionnelle. |
| Vrai logo OpenAI au lieu du cercle CSS | Le prototype n'avait pas d'URL CDN valide ; le SVG existe dans l'app desktop. |
| Les boutons « GitHub » sortent vers le dépôt au lieu de l'ancre `#opensource` | Un bouton qui annonce GitHub et fait défiler la page est une promesse cassée. |
| « Voir sur GitHub » (section Code source) en `text-page` au lieu de `--on-accent` | Le prototype donnait du blanc sur `--ink`, soit **blanc sur blanc en thème sombre** (1,1:1). |
| Boutons « Enregistrer » / « Exporter en PDF » (bande ATS) au lieu d'« Adapter à l'offre » | L'éditeur de CV n'a pas de bouton « Adapter à l'offre » : il enregistre la version et l'exporte en PDF. Le bouton accent garde `--on-accent`, comme tous les autres. |
| Pastille « En attente » en `--page` | Le prototype peignait le statut neutre en `--page` plutôt qu'en `--surface-alt` ; on le garde. Le statut neutre se creuse dans la carte en sombre — c'est voulu. |
| `ThemeToggle` en `useSyncExternalStore` | Le thème est un attribut posé hors de React ; le recopier dans un `useState` depuis un `useEffect` viole `react-hooks/set-state-in-effect`. |
| Pas de route `/api/download/[platform]` | Export statique : aucune route serveur n'est possible. |
| Sur-titre du hero (« Application desktop · Windows · macOS · Linux ») supprimé | Demande de l'auteur. |

### Provenance des assets

- `public/providers/*.svg` — repris de `src/assets/providers/` de l'application desktop.
- `public/brand/*.svg` — extraits du paquet npm `simple-icons` au moment de
  l'intégration, puis vendorisés. Le paquet n'est plus une dépendance : pour ajouter une
  marque, `npx simple-icons@latest` ou copier depuis simpleicons.org.
- `public/logo-candilog*.svg` — fournis par l'auteur. `logo-candilog-dark.svg` n'est pas
  utilisé (le monogramme dégradé tient sur les deux fonds), il est conservé au cas où.
