use super::super::nom_de_fichier_sur;

#[test]
fn un_nom_normal_est_conserve() {
    assert_eq!(
        nom_de_fichier_sur("candilog-ubuntu-0.3.0.deb"),
        "candilog-ubuntu-0.3.0.deb"
    );
}
