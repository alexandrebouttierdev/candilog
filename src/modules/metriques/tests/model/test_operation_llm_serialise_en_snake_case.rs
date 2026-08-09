//! Cas de test isolé.

use super::*;

#[test]
fn test_operation_llm_serialise_en_snake_case() {
    let json = serde_json::to_string(&OperationLlm::AnalyserEntretien).unwrap();
    assert_eq!(json, "\"analyser_entretien\"");
}
