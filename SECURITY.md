# Signaler une vulnérabilité

Merci de **ne pas ouvrir d'issue publique** pour une faille de sécurité : le dépôt est
public, et une issue rend le problème exploitable avant qu'un correctif n'existe.

## Comment signaler

Utiliser l'avis de sécurité privé de GitHub :
[**Report a vulnerability**](https://github.com/alexandrebouttierdev/candilog/security/advisories/new).
Le fil reste privé entre vous et le mainteneur jusqu'à la publication du correctif.

Un signalement utile contient : la version de Candilog (*Réglages → À propos*), le système
d'exploitation, les étapes de reproduction, l'impact constaté, et — si vous en avez une —
une piste de correction.

## Ce à quoi vous pouvez vous attendre

| Étape | Délai visé |
| --- | --- |
| Accusé de réception | 7 jours |
| Première évaluation (confirmé / non reproduit / hors périmètre) | 14 jours |
| Correctif publié pour une faille confirmée | selon la gravité, la mise à jour est annoncée dans la release |

Candilog est un projet indépendant maintenu sur du temps personnel : ces délais sont un
engagement de bonne foi, pas un contrat de service. Il n'existe pas de programme de
récompense.

## Périmètre

Sont dans le périmètre, entre autres :

- exécution de code ou accès à des fichiers depuis une donnée non fiable — texte d'offre,
  PDF importé, réponse d'un fournisseur IA, nom d'asset d'une release ;
- contournement de la vérification d'empreinte de la mise à jour in-app, ou téléchargement
  depuis une origine non autorisée ;
- fuite de la clé API du fournisseur IA hors du trousseau système ;
- lecture ou écriture hors du dossier de données et des emplacements choisis par
  l'utilisateur ;
- corruption ou perte de la base dans un usage normal.

Sont **hors** périmètre :

- l'absence de signature de code Windows et macOS, connue et documentée dans le
  [`README`](./README.md) ;
- ce qu'un fournisseur IA distant fait des données que l'utilisateur a explicitement choisi
  de lui envoyer ;
- un accès physique à une session déverrouillée : la base est protégée par les permissions
  du compte utilisateur, pas par un chiffrement ;
- les avis `unmaintained` sans correctif disponible, listés et justifiés dans
  [`deny.toml`](./deny.toml).

## Versions couvertes

Seule la dernière version publiée reçoit des correctifs. Il n'y a pas de branche de
maintenance pour les versions antérieures.
