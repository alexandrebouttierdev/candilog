use super::super::nom_de_fichier_sur;

#[test]
fn un_nom_dangereux_ou_vide_est_remplace() {
    assert_eq!(nom_de_fichier_sur(""), "candilog-installateur");
    assert_eq!(nom_de_fichier_sur("."), "candilog-installateur");
    assert_eq!(nom_de_fichier_sur(".."), "candilog-installateur");
    assert_eq!(nom_de_fichier_sur("  "), "candilog-installateur");
}
