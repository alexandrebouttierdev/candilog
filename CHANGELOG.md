# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), et le versionnage
[SemVer](https://semver.org/lang/fr/). Chaque version publiée correspond à un tag
`v<version>` et à une [release GitHub](https://github.com/alexandrebouttierdev/candilog/releases).

## [Non publié]

### Modifié

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
