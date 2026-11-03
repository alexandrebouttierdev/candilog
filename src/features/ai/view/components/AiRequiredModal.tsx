import { useNavigate } from "react-router-dom";
import { Button, ModalHost } from "@/shared/ui";
import { useAiRequiredStore } from "../../viewmodel/ai-required-store";
import { PATHS } from "@/shared/lib/paths";

/** Onglet de l'écran IA ouvert par la popup : `?onglet=local` ou `?onglet=fournisseurs`. */
export const AI_TAB_PARAM = "onglet";

/**
 * Invite à configurer l'IA avant toute action qui en dépend.
 *
 * Deux voies, présentées côte à côte parce qu'elles n'engagent pas la même chose : l'IA
 * locale (rien ne quitte l'ordinateur, gratuit, plus lent) ou un fournisseur externe
 * (rapide, payant, les données lui sont envoyées). Chacune ouvre directement l'onglet
 * correspondant de l'écran Intelligence artificielle.
 */
export function AiRequiredModal() {
  const open = useAiRequiredStore((state) => state.open);
  const hide = useAiRequiredStore((state) => state.hide);
  const navigate = useNavigate();
  const go = (tab: "local" | "fournisseurs") => {
    hide();
    void navigate(`${PATHS.ai}?${AI_TAB_PARAM}=${tab}`);
  };

  return (
    <ModalHost
      open={open}
      title="IA non configurée"
      subtitle="Un fournisseur est requis pour cette action"
      cancelLabel="Fermer"
      onClose={hide}
      width="520px"
    >
      <div className="flex flex-col gap-3 pt-3">
        <p className="text-body leading-relaxed text-tx-3">
          Candilog n’a pas encore de fournisseur ni de modèle IA prêts à l’emploi. Choisissez
          comment l’IA doit fonctionner, puis réessayez. Ce qui n’est pas encore enregistré dans
          l’écran ouvert ne sera pas conservé.
        </p>
        <div className="grid grid-cols-2 gap-2">
          <ChoiceCard
            title="Modèle local"
            detail="Installé sur cet ordinateur : rien n’en sort, gratuit, utilisable hors connexion ; plus lent."
            action="Installer l’IA locale"
            onSelect={() => go("local")}
          />
          <ChoiceCard
            title="Fournisseur externe"
            detail="Anthropic, OpenAI, Mistral AI… : plus rapide ; les données de la tâche lui sont envoyées, l’usage est facturé."
            action="Configurer un fournisseur"
            onSelect={() => go("fournisseurs")}
          />
        </div>
      </div>
    </ModalHost>
  );
}

function ChoiceCard({
  title,
  detail,
  action,
  onSelect,
}: {
  title: string;
  detail: string;
  action: string;
  onSelect: () => void;
}) {
  return (
    <div className="flex flex-col gap-2 rounded-r9 border border-bd-soft bg-group p-3">
      <p className="text-entry text-tx">{title}</p>
      <p className="flex-1 text-sub leading-normal text-tx-4">{detail}</p>
      <Button variant="secondary" onClick={onSelect}>
        {action}
      </Button>
    </div>
  );
}
