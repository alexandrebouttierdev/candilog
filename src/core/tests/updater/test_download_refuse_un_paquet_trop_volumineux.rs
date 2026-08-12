use super::super::{verifier_taille_paquet, MAX_UPDATE_BYTES};

#[test]
fn download_refuse_un_paquet_trop_volumineux_avant_de_creer_un_fichier() {
    verifier_taille_paquet(MAX_UPDATE_BYTES).expect("la limite exacte doit etre acceptee");
    let error = verifier_taille_paquet(MAX_UPDATE_BYTES + 1)
        .expect_err("un octet au-dessus de la limite doit etre refuse");
    assert!(error.to_string().contains("taille maximale"));
}
