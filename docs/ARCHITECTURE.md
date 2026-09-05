# Architecture native Candilog

Candilog est une application desktop Tauri 2. React rend l'interface ; Rust porte le métier,
SQLite et l'IA. Le pont IPC est le seul contrat entre les deux côtés.

L'interface suit `docs/DESIGN.md` : ne pas inventer de style en dehors de ce système.

```text
Vue React → ViewModel (hook) → service frontend → invoke
                                                    ↓
                                              Commande Tauri
                                                    ↓
                                              Service métier
                                                    ↓
                                              Repository
                                                    ↓
                                              SQLite
```

- `src/` : frontend feature-first (`model`, `view`, `viewmodel`, `services`), design system
  dans `src/shared/ui/`.
- `src-tauri/src/app/` : état partagé (`AppState`) et démarrage — `bootstrap.rs` enregistre
  toutes les commandes.
- `src-tauri/src/core/` : chemins, base de données, erreurs, journal, pagination,
  sauvegardes, mises à jour, coffre à secrets, validation partagée.
- `src-tauri/src/features/` : domaines métier (`domain`, `application`, `infrastructure`,
  `presentation`). L'IA vit dans `features/ai/` (prompts, providers HTTP, extraction PDF,
  scoring ATS).
- `src-tauri/src/infrastructure/` : PDF d'export (CV et lettres).
- `src-tauri/migrations/` : schéma SQLite embarqué (`init_schema.sql`, `PRAGMA user_version`).

Les accès au système restent natifs : dialogues de fichier (`core/files.rs`) et lecture du
presse-papiers (`core/clipboard.rs`, commande `documents_read_clipboard`). La webview
n'expose ni l'un ni l'autre, et aucune permission large n'est ouverte côté capacités. Elle
n'a pas non plus accès au dossier de données : la photo de profil, écrite dans
`AppPaths::photos_dir`, lui parvient en `data:` URL par `profile_photo` (`docs/DATA.md`).

Le `domain` n'importe ni Tauri ni rusqlite. Les vues n'appellent jamais `invoke` directement :
elles passent par `src/shared/services/ipc.ts`. Les providers IA sont derrière `LlmProvider`.
La clé API reste dans le coffre natif via `keyring`.

## CV ciblé — `ResumeWorkspace`

Après une génération IA, l'éditeur de CV ne dépend plus du profil ni du snapshot
`ResumeGeneration` : Rust compose un **`ResumeWorkspace`** (`schema_version = 1`) contenant
le document complet (`ResumeDocument`), l'offre structurée, l'analyse ATS, le score local
(`MatchScore`), le score initial, une bibliothèque figée des contenus optionnels du profil,
les décisions éditoriales de session, la mesure PDF et les recommandations courantes.

Le profil est la source exhaustive ; le document est une sélection. Identité, coordonnées,
expériences et formations composent le socle initial. Compétences, projets, certifications
et langues restent dans `profile_library` tant que l'utilisateur ne les ajoute pas.

| Commande IPC | Rôle |
| --- | --- |
| `documents_resume_prepare` | Fige profil + `ResumeGeneration` en workspace autonome |
| `documents_resume_recalculate` | Revalide, remesure par le moteur PDF et recalcule score et recommandations locales |
| `documents_resume_apply_proposal` | Applique une proposition puis recalcule |
| `documents_resume_reject_proposal` | Refuse une proposition sans modifier le document, puis recalcule |
| `documents_resume_export_pdf` | Exporte un `ResumeDocument` (plus un `ResumeGeneration`) |

La pertinence sémantique vient de `features/ai` sous forme d'identifiants du catalogue du
profil. Le grounding supprime tout identifiant inventé. Le classement final (maximum quatre),
le filtrage des choix utilisateur, les simulations d'ajout/remplacement et la mesure de place
vivent dans `features/documents/application/resume_workspace.rs`. Chaque simulation utilise
`ResumePdf::measure`, donc les mêmes polices, largeurs, marges, retours à la ligne et paliers
de densité que l'export.

Les exigences absentes du profil restent des écarts informatifs : elles ne deviennent jamais
un bouton d'ajout au CV. Si le fournisseur IA est indisponible, un socle factuel est composé
localement et `recommendation_error` désactive seulement la sélection IA ; la bibliothèque
du profil et l'édition continuent de fonctionner.

Les anciens contenus `ResumeGeneration` restent lisibles en bibliothèque ; la conversion en
workspace n'a lieu qu'à la première ouverture en édition ou à l'export (`prepare_workspace`),
sans réécrire silencieusement la bibliothèque.

## Événements

Les traitements longs remontent leur progression par événements Tauri, émis depuis la
couche `presentation` de la feature concernée :

| Événement | Émetteur |
| --- | --- |
| `ia-progression` | `features/ai` — génération et analyse |
| `profile_import_progress` | `features/ai` — import de profil depuis un CV |
| `update-progress` | `features/settings` — téléchargement d'une mise à jour |

L'écran « Analyser » sépare strictement le choix du PDF du traitement :
`ai_select_resume_file` ouvre le dialogue natif et retourne le nom et le chemin validé,
puis seule l'action explicite « Analyser le CV » appelle `ai_analyze_resume`. Le chemin est
revalidé en Rust avant lecture. L'import du profil conserve un sélecteur intégré à sa
commande ; son premier événement n'est émis qu'une fois le fichier choisi afin de ne jamais
annoncer une analyse qui n'a pas commencé.

Le cycle de vie des traitements IA est coordonné dans `features/ai/viewmodel`. La coque
`AppShell` porte l'unique garde React Router : analyse de CV, génération de CV, lettre et
import de profil ne dupliquent donc pas la confirmation de sortie. Après une demande
d'arrêt, l'identifiant est invalidé côté interface avant l'IPC et les résultats tardifs sont
ignorés ; `ai_cancel` déclenche ensuite le `CancellationToken` Rust qui abandonne le futur
HTTP en cours.

`core/events/` reste un module de réservation, sans contenu.

## Le site

`website/` est le site public candilog.fr : un projet Next.js **autonome**, sans lien de
code avec l'application. Il ne partage ni dépendances, ni configuration TypeScript, ni
design system — voir `website/README.md` et `website/DESIGN.md`.

## Documents liés

`docs/CODE_RULES.md` (règles de code), `docs/DATA.md` (schéma et données),
`docs/AI.md` (IA), `docs/DESIGN.md` (interface), `docs/DEVELOPMENT.md` (commandes).
