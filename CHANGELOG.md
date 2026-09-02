# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), et le versionnage
[SemVer](https://semver.org/lang/fr/). Chaque version publiée correspond à un tag
`v<version>` et à une [release GitHub](https://github.com/alexandrebouttierdev/candilog/releases).

## [Non publié]

Rien pour l'instant.

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
- Fournisseurs IA au choix : Ollama (local), Claude, OpenAI, Gemini, Mistral, Nvidia ou
  point de terminaison personnalisé. La clé API vit dans le coffre du système.
- Sauvegarde et restauration de la base, avec retour arrière en cas d'échec.
- Mise à jour assistée depuis les GitHub Releases : empreinte SHA-256 vérifiée avant
  l'ouverture de l'installateur, jamais d'installation silencieuse.
