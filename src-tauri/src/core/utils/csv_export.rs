//! Règles communes aux exports CSV destinés à un tableur.

/// Marque d'ordre d'octets UTF-8.
///
/// Excel, en double-clic, ne détecte pas l'UTF-8 sans elle et décode en ANSI : « Société »
/// s'affiche « SociÃ©tÃ© », c'est-à-dire la quasi-totalité d'un export français. Le
/// point-virgule séparateur vise déjà ce tableur ; sans la marque, l'intention s'arrête à
/// mi-chemin.
const BOM: &str = "\u{feff}";

/// Caractères par lesquels un tableur reconnaît le début d'une formule.
const DEBUTS_DE_FORMULE: [char; 4] = ['=', '+', '-', '@'];

/// Neutralise un champ que le tableur interpréterait comme une formule.
///
/// Le contenu vient de l'utilisateur, mais pas seulement de lui : une offre collée dans les
/// notes, un intitulé extrait par l'IA. Un champ ouvrant par `=` devient une formule à
/// l'ouverture du fichier, y compris chez la personne à qui l'export est transmis. Le
/// préfixe apostrophe est la parade reconnue par Excel et LibreOffice ; il n'apparaît pas
/// dans la cellule affichée.
#[must_use]
pub fn champ_sur(valeur: &str) -> String {
    if valeur.starts_with(DEBUTS_DE_FORMULE) {
        format!("'{valeur}")
    } else {
        valeur.to_owned()
    }
}

/// Préfixe un CSV de la marque d'ordre d'octets attendue par Excel.
#[must_use]
pub fn avec_bom(csv: &str) -> String {
    if csv.starts_with(BOM) {
        return csv.to_owned();
    }
    format!("{BOM}{csv}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_bom_precede_le_contenu_et_ne_se_dedouble_pas() {
        let csv = avec_bom("poste;entreprise\n");
        assert!(csv.starts_with('\u{feff}'));
        assert_eq!(avec_bom(&csv), csv);
        assert!(csv.ends_with("poste;entreprise\n"));
    }

    #[test]
    fn un_champ_ouvrant_une_formule_est_neutralise() {
        assert_eq!(champ_sur("=1+1"), "'=1+1");
        assert_eq!(champ_sur("+33 6 12 34 56 78"), "'+33 6 12 34 56 78");
        assert_eq!(champ_sur("-5"), "'-5");
        assert_eq!(champ_sur("@ Nova"), "'@ Nova");
    }

    #[test]
    fn un_champ_ordinaire_traverse_sans_modification() {
        assert_eq!(champ_sur("Développeuse Rust"), "Développeuse Rust");
        assert_eq!(champ_sur(""), "");
        assert_eq!(champ_sur("2026-08-30"), "2026-08-30");
    }
}
