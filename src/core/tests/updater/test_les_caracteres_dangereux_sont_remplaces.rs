use super::super::nom_de_fichier_sur;

#[test]
fn les_caracteres_dangereux_sont_remplaces() {
    assert_eq!(
        nom_de_fichier_sur("a/b\\c:d*e?f\"g<h>i|j"),
        "a_b_c_d_e_f_g_h_i_j"
    );
}
