use super::*;

#[test]
fn la_grille_propose_sept_fournisseurs() {
    let grille = providers();
    assert_eq!(grille.len(), 7);
    assert_eq!(
        grille.iter().map(provider_label).collect::<Vec<_>>(),
        [
            "Ollama",
            "Claude",
            "OpenAI",
            "Gemini",
            "Mistral",
            "NVIDIA",
            "Personnalisé",
        ]
    );
}
