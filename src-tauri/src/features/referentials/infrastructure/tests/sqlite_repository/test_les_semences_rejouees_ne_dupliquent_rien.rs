//! Rejouer `init_schema.sql` sur une base déjà initialisée ne doit rien dupliquer.

use super::*;

#[test]
fn un_second_demarrage_laisse_les_referentiels_inchanges() {
    let (repo, pool) = context();
    let avant = repo.load().unwrap();

    replay_schema(&pool);

    let apres = repo.load().unwrap();
    assert_eq!(avant.sectors, apres.sectors);
    assert_eq!(avant.professional_domains, apres.professional_domains);
    assert_eq!(avant.company_types, apres.company_types);
    assert_eq!(avant.contract_types, apres.contract_types);
}
