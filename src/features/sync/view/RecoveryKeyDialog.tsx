import { useState } from "react";
import { ConfirmDialog } from "@/shared/ui";

/**
 * Clé de récupération, affichée **une seule fois** à la création de l'espace. Elle remplace
 * la phrase secrète oubliée ; sans l'une ni l'autre, l'espace distant est illisible, y
 * compris pour Candilog. Le dialogue ne se ferme qu'après confirmation explicite.
 */
export function RecoveryKeyDialog({ recoveryKey, onClose }: { recoveryKey: string; onClose: () => void }) {
  const [kept, setKept] = useState(false);
  return (
    <ConfirmDialog
      open
      register="information"
      title="Votre clé de récupération"
      description="Si vous oubliez votre phrase secrète, cette clé est le seul moyen d'ouvrir l'espace depuis un nouvel ordinateur. Conservez-la hors de cet ordinateur : sur papier ou dans un gestionnaire de mots de passe."
      footnote="affichée une seule fois"
      confirmLabel="J'ai conservé ma clé"
      hideCancel
      confirmDisabled={!kept}
      dismissDisabled={!kept}
      onCancel={onClose}
      onConfirm={onClose}
    >
      <p
        aria-label="Clé de récupération"
        className="mb-3 rounded-r8 bg-group px-3 py-3 text-center font-mono text-ui tracking-wide break-all text-tx select-all"
      >
        {recoveryKey}
      </p>
      <label className="flex items-center gap-2 text-ui text-tx-3">
        <input type="checkbox" checked={kept} onChange={(event) => setKept(event.target.checked)} />
        J'ai noté cette clé en lieu sûr.
      </label>
    </ConfirmDialog>
  );
}
