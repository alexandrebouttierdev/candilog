# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), et le versionnage
[SemVer](https://semver.org/lang/fr/). Chaque version publiée correspond à un tag
`v<version>` et à une [release GitHub](https://github.com/alexandrebouttierdev/candilog/releases).

## [Non publié]

### Modifié

- Refonte v2 (en cours) — nouvelle coque : barre de titre avec fil d'Ariane et onglets de
  vue, navigation à six destinations (Aujourd'hui, Candidatures, Relations, Documents,
  Intelligence artificielle, Profil) avec décomptes, barre d'état rappelant le contrat
  clavier de l'écran. Les Réglages deviennent une surcouche (`⌘,`) en six sections ;
  Apparence regroupe thème, densité des listes, animations et son de fin de traitement.
- Refonte v2 — palette de commandes `⌘K` et navigation au clavier (`G` puis `A/C/R/D/P`).
- Refonte v2 — nouvelle palette de couleurs, titres en IBM Plex Serif, données en IBM Plex
  Mono ; dialogues à trois registres qui énumèrent les conséquences ; une seule notification
  à la fois ; fenêtre utilisable dès 940 × 560 px, navigation réduite sous 1060 px.

- Refonte v2 — Candidatures : chaque candidature reçoit une référence lisible (`CAN-142`) et
  un canal « Trouvée via » (Offre, Site, Réseau, Spontanée) ; les candidatures existantes
  sont numérotées dans leur ordre de création. La vue Liste devient une liste groupée par
  statut (Refusée repliée), avec échéance en pastille (entretien, relance, silence de plus
  de 14 jours) et colonnes selon la largeur disponible. Nouvel inspecteur avec historique
  des statuts, menu d'actions (clic droit ou `⋯`), dialogue de suppression qui énumère
  relances, entretiens et historique emportés, duplication (`⌘D`), programmation d'une
  relance (`R`), changement de statut (`S`). Le formulaire garde toutes les précisions dans
  « Plus de détails » et demande confirmation avant de fermer une fiche modifiée.
- Export CSV des candidatures : colonnes `reference` et `canal` ajoutées.
- Le lien de l'offre n'est plus exigé que pour une offre publiée ; il devient facultatif
  pour une candidature trouvée sur le site de l'entreprise ou par le réseau.

- Lettres de motivation : rédaction LLM à partir d’un pack d’évidences courtes (plus de collage template du CV), nettoyage d’offre (codes REC, slogans, process RH), grounding factuel Rust (zéro invention). Fallback template si brouillon trop court.

- Score ATS : reclassement automatique des exigences soft (Agile, qualité, MOA, UX, IA…) hors compétences dures pour éviter la dilution (~31/100) ; famille Java/Spring pour crédit partiel.

- Score ATS : soft skills / mots-clés / proximité métier en **bonus** (plus en moyenne avec des 0 qui écrasaient vers ~17/100) ; équivalence CI/CD ↔ GitLab CI / intégration continue ; années aussi déduites des plages de dates.

- Score ATS multi-métiers : matching exact / transférable (familles d'outils génériques),
  corpus candidat élargi (projets, certifications, langues), proximité métier en bonus,
  mots-clés dédupliqués des compétences, prompts offre / recommandations plus stricts
  (exigences vs contexte entreprise, pas d'invention). Tests Open + multi-domaines.

- Score ATS : exigences structurées par catégorie, importance et caractère obligatoire ;
  pondération dynamique, détail explicable par catégorie, correspondances exactes,
  équivalentes, transférables ou partielles, et plafonnement des qualifications
  réglementaires obligatoires absentes. Le même calcul suit les extractions Vision et Texte.

- Analyse de CV : recommandations reliées à une exigence et à des preuves du CV ; les
  reformulations répétitives ou ajoutant une exigence absente sont écartées, et les écarts
  restent informatifs sans action d'ajout.

- Analyse de CV : score déterministe enrichi avec le texte PDF brut (plus seulement le JSON LLM) ; mode Vision si le modèle le permet, avec repli Texte.

- Analyse de CV : ne plus scorer l'expérience à 0 quand les dates manquent ;
  déduction du titre et des années depuis le texte ; filtre des compétences
  issues du blurb entreprise (ex. « Expertises reconnues ») hors exigences poste.

- Analyse de CV : padding horizontal autour du nom de fichier ; marge sous « Offre ciblée ».

- Import CV : descriptions de compétences et de projets mieux récupérées (clés
  alternatives `detail` / `resume`, projets en chaîne, séparation d'une description
  collée dans le libellé avant le recadrage sur le texte du CV).

- Réglages IA / Configuration : liste de modèles compacte avec logo du fournisseur ;
  bouton « Tester » (benchmark) à côté de « Tester la connexion », désactivé sans modèle.

- Réglages IA : le bandeau d'état (fournisseur / modèle / Configuré) en tête de page
  est retiré. « Tester la connexion » est aligné à droite du titre de la carte
  Configuration.

- Top bar : conserve le sélecteur de modèle, « Tester » et le raccourci réglages IA ;
  les notes / recherches / autres actions contextuelles (Profil, Analyses, Documents,
  Aujourd'hui, Réglages, Calendrier) quittent le bandeau. Mes lettres a une recherche
  in-page alignée sur Mes CV. Les onglets Profil ne forcent plus le scroll horizontal
  de la page.

- Import de profil depuis un CV : dates de fin découpées depuis une plage collée dans
  `start_date`, descriptions d'expériences / projets / certifications mieux conservées
  (recadrage par fragments ; en Vision, les textes libres et le prénom / nom lus sur les
  images ne sont plus effacés par un PDF complémentaire partiel). Un nom complet coincé
  dans un seul champ est découpé. Compétences et certifications acceptent une description
  facultative. Mesure locale : voir `src-tauri/examples/cv_import_baseline.md`
  et `src-tauri/examples/PROMPT_IMPORT_CV.md`.

- Page Profil : sections Identité, Objectif professionnel et Présence en ligne en onglets
  dédiés (modales séparées). Le bouton « Modifier le profil » et le panneau latéral
  Identité sont retirés. Le bloc « Réinitialiser le profil » est aligné en haut à droite
  du bandeau, visible quel que soit l'onglet.

- Benchmark « Tester l'IA » : le score Formations accepte les libellés paraphrasés ou
  partiels (diplôme / école), aligné sur ce que l'import affiche déjà.

- Fournisseur IA NVIDIA remplacé par DeepSeek (API compatible OpenAI,
  `https://api.deepseek.com`, modèle par défaut `deepseek-v4-flash`). Un réglage encore
  enregistré sous `nvidia` est relu comme DeepSeek.

### Retiré

- Tri par colonne de la table des candidatures : la liste v2 est groupée par statut, les
  plus récentes d'abord.
- Tour d'accueil : le premier lancement ouvre directement Aujourd'hui, dont l'état vide
  propose les premières actions.

## [0.0.1] — non publiée

Première version. Aucune release n'a encore été publiée : ce numéro est celui que portera
la première.

### Ajouté

- Suivi des candidatures en kanban ou en table, filtres et recherche exécutés en base,
  historique de statut, export CSV.
- Répertoire d'entreprises et de contacts, avec héritage des valeurs de l'entreprise
  (ville, adresse, type) sur la candidature.
- Calendrier des entretiens et des relances.
- Profil professionnel, génération de CV et de lettres de motivation en PDF A4 d'une page,
  analyse ATS déterministe.
- IA locale embarquée : moteur d'inférence llama.cpp lié au binaire, quatre profils de
  modèles GGUF installables depuis les réglages, téléchargés depuis Hugging Face et vérifiés
  par empreinte SHA-256 avant usage. Aucun serveur ni outil externe à installer.
- Autres fournisseurs IA au choix : Ollama (local), Claude, OpenAI, Gemini, Mistral,
  DeepSeek ou point de terminaison personnalisé. La clé API vit dans le coffre du système.
- Sauvegarde et restauration de la base, avec retour arrière en cas d'échec.
- Mise à jour assistée depuis les GitHub Releases : empreinte SHA-256 vérifiée avant
  l'ouverture de l'installateur, jamais d'installation silencieuse.
- Attestation de provenance Sigstore sur chaque binaire publié, vérifiable par
  `gh attestation verify`.
- Licences des composants redistribués (polices IBM Plex, Material Symbols, llama.cpp)
  livrées avec chaque paquet, sous `licenses/`.
