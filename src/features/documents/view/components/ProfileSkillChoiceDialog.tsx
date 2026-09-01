import { useState } from "react";
import { ErrorBanner, ModalHost } from "@/shared/ui";
import type { PendingProfileSkill } from "../../viewmodel/useResumeEditor";

/**
 * Confirmation séparée de l'ajout au profil, après l'acceptation d'une compétence manquante.
 *
 * Réutilise `ModalHost` plutôt qu'une nouvelle superposition : « CV uniquement » referme la
 * demande sans toucher au profil (bouton d'annulation de la modale), « Ajouter au profil »
 * appelle le service et met à jour `PROFILE_KEY` (délégué à `useResumeEditor`). Un échec
 * laisse la compétence acceptée au CV, garde la modale ouverte et affiche l'erreur avec un
 * bouton Réessayer, sans appeler le service deux fois.
 */
export function ProfileSkillChoiceDialog({
  pending,
  error,
  onKeepResumeOnly,
  onAddToProfile,
}: {
  pending: PendingProfileSkill | null;
  error: string | null;
  onKeepResumeOnly: () => void;
  onAddToProfile: () => Promise<void>;
}) {
  const [busy, setBusy] = useState(false);

  const ajouter = async () => {
    setBusy(true);
    try {
      await onAddToProfile();
    } finally {
      setBusy(false);
    }
  };

  return (
    <ModalHost
      open={pending !== null}
      icon="person_add"
      title="Ajouter cette compétence au profil ?"
      subtitle={pending?.skill}
      cancelLabel="CV uniquement"
      submitLabel="Ajouter au profil"
      submitIcon="person_add"
      busy={busy}
      onClose={onKeepResumeOnly}
      onSubmit={() => void ajouter()}
    >
      <div className="space-y-3 pt-3">
        <p className="text-body text-ink-muted">
          « {pending?.skill} » vient d’être ajoutée à ce CV. L’ajouter aussi à votre profil
          général la rendra disponible pour vos prochains CV.
        </p>
        {error ? (
          <ErrorBanner
            title="Ajout au profil impossible"
            message={error}
            onRetry={() => void ajouter()}
          />
        ) : null}
      </div>
    </ModalHost>
  );
}
