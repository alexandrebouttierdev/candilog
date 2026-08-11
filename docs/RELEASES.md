# Releases natives

Les releases ciblent Linux, Windows et macOS sans Node. Le dépôt public `candilog-releases` garde `latest.json` et les paquets signés. Le client compare SemVer, télécharge **sous le dossier de données** (sous-dossier `mises-à-jour`, et non `/tmp`, que beaucoup de systèmes purgent au redémarrage), puis vérifie la signature minisign avec la clé publique embarquée. Un paquet dont la signature ne correspond pas est supprimé.

La mise à jour est **assistée, pas automatique** : l'installation par plateforme et le redémarrage ne sont pas implémentés. Une fois le paquet vérifié, Candilog indique où il se trouve et l'utilisateur l'installe comme toute application de son système. Cette limite est délibérément énoncée ici plutôt que promise : annoncer une chaîne complète qui s'arrête à mi-parcours laisse l'utilisateur avec un fichier dont il ne sait que faire.

Une mise à jour dont la signature ne correspond pas doit être supprimée et refusée. Linux distribue AppImage/deb/rpm, Windows un installateur, macOS une archive d'application. Les certificats de signature de code restent distincts de minisign.

