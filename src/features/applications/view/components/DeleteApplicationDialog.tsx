import type { Application } from "@/shared/types/generated/applications";
import { ConfirmDialog } from "@/shared/ui";
import type { DialogConsequence } from "@/shared/ui";
import { formatReference } from "../../model/presentation";
import { useDeletionImpact } from "../../viewmodel/useApplicationDetails";

function pluriel(count: number, singulier: string, plurielForme: string): string {
  return count > 1 ? plurielForme : singulier;
}

/**
 * Dialogue `del-cand` (`INTERACTIONS.md` §5) : il énumère ce qui disparaît — relances,
 * entretiens, historique, comptés en base — et dit ce qui survit : l'entreprise et le
 * contact. Registre destruction, focus initial sur « Annuler ».
 */
export function DeleteApplicationDialog({
  application,
  busy,
  onCancel,
  onConfirm,
}: {
  application: Application | null;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const impact = useDeletionImpact(application?.id ?? null);
  const data = impact.data;
  const consequences: DialogConsequence[] = data
    ? [
        { label: pluriel(data.follow_ups, "Relance programmée", "Relances programmées"), value: String(data.follow_ups) },
        { label: pluriel(data.interviews, "Entretien", "Entretiens"), value: String(data.interviews) },
        {
          label: pluriel(data.status_changes, "Étape de l'historique", "Étapes de l'historique"),
          value: String(data.status_changes),
        },
      ].filter((item) => item.value !== "0")
    : [];

  return (
    <ConfirmDialog
      open={application !== null}
      title={application ? `Supprimer ${formatReference(application.reference_number)} ?` : ""}
      description={
        application
          ? `« ${application.job_title} » chez ${application.company_name ?? "cette entreprise"} disparaît de votre suivi, avec ses échéances.`
          : ""
      }
      consequences={consequences}
      note="L'entreprise et le contact associés sont conservés."
      footnote="action définitive"
      confirmLabel="Supprimer"
      busy={busy}
      onCancel={onCancel}
      onConfirm={onConfirm}
    />
  );
}
