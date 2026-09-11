//! Helpers communs et déclaration des cas de test.
use super::*;

mod test_is_local_or_private_ip_distingue_adresses;
mod test_is_local_or_private_ip_refuse_ipv4_mappee_en_ipv6;
#[path = "test_is_valid_email_accepte_une_adresse_courante.rs"]
mod test_is_valid_email_accepte_une_adresse_courante;
#[path = "test_is_valid_email_refuse_un_domaine_sans_point.rs"]
mod test_is_valid_email_refuse_un_domaine_sans_point;
#[path = "test_validate_optional_email_accepte_vide.rs"]
mod test_validate_optional_email_accepte_vide;
mod test_validate_optional_http_url_accepte_https;
mod test_validate_optional_http_url_refuse_javascript;
mod test_validate_user_file_path_refuse_la_traversee;
