# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), et le versionnage
[SemVer](https://semver.org/lang/fr/). Chaque version publiée correspond à un tag
`v<version>` et à une [release GitHub](https://github.com/alexandrebouttierdev/candilog/releases).

## [Non publié]

### Modifié

- Import de profil depuis un CV : le texte est lu dans l'ordre de mise en page
  (`pdftotext -layout`, repli sur l'extracteur de flux), puis un e-mail ou un
  téléphone vide est recopié seulement s'il est déjà dans ce texte, et une
  formation manquante seulement si le diplôme ou l'établissement est unique et
  exact. Mesure locale : voir `src-tauri/examples/cv_import_baseline.md`.

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
