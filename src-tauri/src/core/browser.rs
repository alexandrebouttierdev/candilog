//! Ouverture d'une URL dans le navigateur du système.

use crate::core::errors::{AppError, AppResult};

/// Longueur maximale acceptée, très au-delà d'une URL d'offre réelle.
const LONGUEUR_MAX: usize = 2048;

/// Ouvre une URL externe après validation.
///
/// La capability `opener:allow-open-url` reste volontairement restreinte à deux origines
/// (`docs/CODE_RULES.md` §10) : une offre d'emploi, un site d'entreprise ou un profil
/// LinkedIn ne peuvent donc pas être ouverts depuis React, et un `<a target="_blank">` ne
/// fait rien dans la WebView. La commande valide ici le schéma plutôt que d'élargir la
/// portée à `https://*`, ce qui aurait laissé passer `javascript:` ou `file:` par un champ
/// de saisie.
///
/// # Errors
/// Retourne une erreur si l'URL est vide, trop longue, mal formée, d'un schéma autre que
/// `http`/`https`, sans hôte, ou si le lanceur système refuse de démarrer.
pub fn ouvrir_url(url: &str) -> AppResult<()> {
    let valide = valider(url)?;
    tauri_plugin_opener::open_url(&valide, None::<&str>)
        .map_err(|error| AppError::Validation(format!("Ouverture du lien impossible : {error}")))
}

/// Vérifie qu'une URL est ouvrable, et la renvoie normalisée.
fn valider(url: &str) -> AppResult<String> {
    let brut = url.trim();
    if brut.is_empty() {
        return Err(AppError::Validation("Lien vide.".into()));
    }
    if brut.len() > LONGUEUR_MAX {
        return Err(AppError::Validation("Lien trop long.".into()));
    }
    let analysee = url::Url::parse(brut)
        .map_err(|_| AppError::Validation(format!("Lien illisible : {brut}")))?;
    if !matches!(analysee.scheme(), "http" | "https") {
        return Err(AppError::Validation(
            "Seuls les liens http et https sont ouverts.".into(),
        ));
    }
    if analysee.host_str().is_none_or(str::is_empty) {
        return Err(AppError::Validation("Lien sans domaine.".into()));
    }
    Ok(analysee.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepte_une_offre_https() {
        assert_eq!(
            valider("https://exemple.test/offre/42").unwrap(),
            "https://exemple.test/offre/42"
        );
    }

    #[test]
    fn accepte_http_et_normalise() {
        assert_eq!(
            valider("  http://exemple.test  ").unwrap(),
            "http://exemple.test/"
        );
    }

    #[test]
    fn refuse_un_schema_non_web() {
        for lien in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,<script>",
        ] {
            assert!(
                matches!(valider(lien), Err(AppError::Validation(_))),
                "{lien} aurait dû être refusé"
            );
        }
    }

    #[test]
    fn refuse_un_lien_vide_ou_illisible() {
        assert!(matches!(valider("   "), Err(AppError::Validation(_))));
        assert!(matches!(
            valider("pas une url"),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn refuse_un_lien_trop_long() {
        let long = format!("https://exemple.test/{}", "a".repeat(LONGUEUR_MAX));
        assert!(matches!(valider(&long), Err(AppError::Validation(_))));
    }
}
