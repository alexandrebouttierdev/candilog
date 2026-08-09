//! Cas de test isolé.

use super::*;

#[test]
fn test_parse_llm_json_repare_les_cles_sans_guillemets() {
    let parsed: ParsedOffer = parse_llm_json(
            r#"{title:"Développeur Golang",skills:["Golang" "Python"],soft_skills:[],experience:"Bac+5",keywords:["microservices"]}"#,
        )
        .unwrap();

    assert_eq!(parsed.title, "Développeur Golang");
    assert_eq!(parsed.skills, vec!["Golang", "Python"]);
}
