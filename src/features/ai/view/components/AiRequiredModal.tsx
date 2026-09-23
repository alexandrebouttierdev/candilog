import { useNavigate } from "react-router-dom";
import { ModalHost } from "@/shared/ui";
import { useAiRequiredStore } from "../../viewmodel/ai-required-store";
import { PATHS } from "@/shared/lib/paths";

/** Invite à configurer l'IA avant toute action qui en dépend. */
export function AiRequiredModal() {
  const open = useAiRequiredStore((state) => state.open);
  const hide = useAiRequiredStore((state) => state.hide);
  const navigate = useNavigate();

  return (
    <ModalHost
      open={open}
      icon="smart_toy"
      title="IA non configurée"
      subtitle="Un fournisseur est requis pour cette action"
      cancelLabel="Fermer"
      submitLabel="Configurer l’IA"
      submitIcon="settings"
      onClose={hide}
      onSubmit={() => {
        hide();
        void navigate(PATHS.ai);
      }}
      width="480px"
    >
      <p className="pt-3 text-body leading-relaxed text-ink-muted">
        Candilog n’a pas encore de fournisseur ni de modèle IA prêts à l’emploi. Configurez
        l'IA locale, Ollama ou un fournisseur distant dans les réglages, puis réessayez.
      </p>
    </ModalHost>
  );
}
