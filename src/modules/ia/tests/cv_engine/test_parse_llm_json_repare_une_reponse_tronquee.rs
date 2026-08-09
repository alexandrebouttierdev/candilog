//! Cas de test isolé.

use super::*;

#[test]
fn test_parse_llm_json_repare_une_reponse_tronquee() {
    let parsed: ParsedOffer = parse_llm_json(
            r#"{"title":"Développeur Golang","skills":["Golang","Python"],"soft_skills":[],"experience":"Bac+5","keywords":["microservices""#,
        )
        .unwrap();

    assert_eq!(parsed.keywords, vec!["microservices"]);
}
