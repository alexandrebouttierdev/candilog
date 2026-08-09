//! Cas de test isolé.

use super::*;

#[test]
fn test_operation_llm_depuis_str_inconnu_est_none() {
    assert_eq!(OperationLlm::depuis_str("inexistant"), None);
}
