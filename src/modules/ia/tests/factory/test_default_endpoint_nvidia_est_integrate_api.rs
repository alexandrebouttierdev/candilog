//! Cas de test isolé.

use super::*;

#[test]
fn test_default_endpoint_nvidia_est_integrate_api() {
    assert_eq!(
        default_endpoint(&ProviderKind::Nvidia),
        "https://integrate.api.nvidia.com"
    );
}
