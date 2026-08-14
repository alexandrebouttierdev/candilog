use super::*;

#[test]
fn le_libelle_couvre_les_sept_fournisseurs() {
    assert_eq!(provider_label(&ProviderKind::Ollama), "Ollama");
    assert_eq!(provider_label(&ProviderKind::Claude), "Claude");
    assert_eq!(provider_label(&ProviderKind::OpenAI), "OpenAI");
    assert_eq!(provider_label(&ProviderKind::Gemini), "Gemini");
    assert_eq!(provider_label(&ProviderKind::Mistral), "Mistral");
    assert_eq!(provider_label(&ProviderKind::Nvidia), "NVIDIA");
    assert_eq!(
        provider_label(&ProviderKind::Custom("personnalisé".into())),
        "Personnalisé"
    );
}
