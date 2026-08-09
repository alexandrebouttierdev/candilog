//! Cas de test isolé.

use super::*;

#[test]
fn test_operation_llm_as_str_et_depuis_str_font_un_aller_retour() {
    for op in [
        OperationLlm::ParseOffer,
        OperationLlm::GenerateCv,
        OperationLlm::AnalyzeAts,
        OperationLlm::ParseCv,
        OperationLlm::AnalyserEntretien,
        OperationLlm::CoverLetter,
    ] {
        assert_eq!(OperationLlm::depuis_str(op.as_str()), Some(op));
    }
}
