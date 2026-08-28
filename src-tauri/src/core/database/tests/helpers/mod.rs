//! Helpers communs et déclaration des cas de test.
use super::*;

/// Enum de substitution aux enums du domaine.
///
/// Les helpers `enum_depuis_texte` / `texte_depuis_enum` sont génériques : les éprouver avec
/// un enum métier réel ferait dépendre le socle d'une feature, alors que ce qui est testé est
/// l'aller-retour `serde` lui-même. Cet enum reproduit la convention effectivement utilisée en
/// base (`SCREAMING_SNAKE_CASE` pour `candidatures.statut`).
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum StatutFactice {
    EnAttente,
}

mod test_enum_depuis_texte_valeur_connue_retourne_la_variante;
mod test_enum_depuis_texte_valeur_inconnue_retourne_erreur;
mod test_maintenant_iso_produit_un_horodatage_rfc3339;
mod test_texte_depuis_enum_restitue_la_valeur_serialisee;
mod test_traduire_contrainte_ligne_absente_retombe_sur_le_label_de_ressource;
mod test_traduire_contrainte_violation_retourne_la_phrase_destinee_a_l_utilisateur;
mod test_traduire_erreur_ligne_absente_retourne_not_found;
mod test_traduire_erreur_violation_de_cle_etrangere_retourne_validation;
