import { useState } from "react";
import { ModalHost } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { AppError } from "@/shared/types/app-error";
import type { RemoteProbe, SetupChoice, SetupOutcome, SetupRequest } from "@/shared/types/generated/sync";
import { formatMoment, PROVIDER_LABELS } from "../model/syncStatus";

type Submit = (request: SetupRequest) => Promise<SetupOutcome>;

function message(error: unknown): string {
  return error instanceof AppError ? error.message : "La mise en service a échoué.";
}

export function SyncSetupDialog({
  probe,
  busy,
  onSubmit,
  onClose,
}: {
  probe: RemoteProbe;
  busy: boolean;
  onSubmit: Submit;
  onClose: () => void;
}) {
  return probe.space_exists ? (
    <JoinSpaceDialog probe={probe} busy={busy} onSubmit={onSubmit} onClose={onClose} />
  ) : (
    <CreateSpaceDialog probe={probe} busy={busy} onSubmit={onSubmit} onClose={onClose} />
  );
}

function Location({ probe }: { probe: RemoteProbe }) {
  return (
    <p className="mb-3 rounded-r8 bg-group px-3 py-2 text-sub text-tx-4">
      <span className="text-tx-2">{PROVIDER_LABELS[probe.provider]}</span>
      {probe.provider === "folder" ? (
        <span className="block font-mono text-caps break-all text-tx-5">{probe.location}</span>
      ) : null}
      {probe.space_created_at ? (
        <span className="block">Espace créé le {formatMoment(probe.space_created_at)}</span>
      ) : null}
    </p>
  );
}

function CreateSpaceDialog({
  probe,
  busy,
  onSubmit,
  onClose,
}: {
  probe: RemoteProbe;
  busy: boolean;
  onSubmit: Submit;
  onClose: () => void;
}) {
  const [error, setError] = useState<string | null>(null);
  const submit = async () => {
    setError(null);
    try {
      await onSubmit({ choice: "create_space" });
    } catch (caught) {
      setError(message(caught));
    }
  };

  return (
    <ModalHost
      open
      title="Créer l'espace de synchronisation"
      subtitle="Les données de cet ordinateur y seront envoyées sans chiffrement."
      submitLabel="Créer et synchroniser"
      busy={busy}
      submitDisabled={busy}
      width="500px"
      onClose={onClose}
      onSubmit={() => void submit()}
    >
      <Location probe={probe} />
      <p className="mb-3 text-sub leading-[1.5] text-tx-4">
        Le fournisseur de stockage et toute personne ayant accès à cet emplacement pourront lire les candidatures,
        les CV, les lettres et les fichiers joints synchronisés.
      </p>
      {probe.legacy_space_exists ? (
        <p className="mb-3 rounded-r8 border border-bd-soft px-3 py-2 text-sub leading-[1.5] text-tx-4">
          Un ancien espace chiffré existe ici. Il restera intact et ne sera ni migré ni utilisé par ce nouvel espace.
        </p>
      ) : null}
      {error ? <p role="alert" className="text-sub text-st-c">{error}</p> : null}
    </ModalHost>
  );
}

const CHOICES: { value: SetupChoice; title: string; detail: string }[] = [
  {
    value: "join_replace_local",
    title: "Récupérer les données de l'espace",
    detail:
      "Les données de cet ordinateur sont remplacées par celles de l'espace. Une sauvegarde complète est prise avant, dans le dossier des exports.",
  },
  {
    value: "join_merge",
    title: "Fusionner avec les données de cet ordinateur",
    detail:
      "Les deux ensembles sont réunis. Une fiche modifiée des deux côtés vous sera présentée pour arbitrage ; rien n'est écrasé en silence.",
  },
];

function JoinSpaceDialog({
  probe,
  busy,
  onSubmit,
  onClose,
}: {
  probe: RemoteProbe;
  busy: boolean;
  onSubmit: Submit;
  onClose: () => void;
}) {
  const [choice, setChoice] = useState<SetupChoice | null>(probe.local_has_data ? null : "join_merge");
  const [error, setError] = useState<string | null>(null);
  const submit = async () => {
    if (!choice) {
      setError("Choisissez ce que deviennent les données de cet ordinateur.");
      return;
    }
    setError(null);
    try {
      await onSubmit({ choice });
    } catch (caught) {
      setError(message(caught));
    }
  };

  return (
    <ModalHost
      open
      title="Rejoindre l'espace existant"
      subtitle="Les données sont synchronisées sans chiffrement."
      submitLabel="Rejoindre"
      busy={busy}
      submitDisabled={busy}
      width="520px"
      onClose={onClose}
      onSubmit={() => void submit()}
    >
      <Location probe={probe} />
      {probe.local_has_data ? (
        <fieldset className="mb-3">
          <legend className="mb-[5px] text-sub font-mid text-tx-4">Cet ordinateur contient déjà des données</legend>
          <div className="flex flex-col gap-1.5">
            {CHOICES.map((item) => (
              <label
                key={item.value}
                className={cn(
                  "flex cursor-pointer items-start gap-2.5 rounded-r8 border px-3 py-2.5",
                  choice === item.value ? "border-ac bg-ac-soft" : "border-bd-menu",
                )}
              >
                <input
                  type="radio"
                  name="sync-setup-choice"
                  value={item.value}
                  checked={choice === item.value}
                  onChange={() => setChoice(item.value)}
                  className="mt-[3px] accent-[var(--ac)]"
                />
                <span>
                  <span className="block text-ui font-medium text-tx-2">{item.title}</span>
                  <span className="block text-sub leading-[1.45] text-tx-5">{item.detail}</span>
                </span>
              </label>
            ))}
          </div>
        </fieldset>
      ) : (
        <p className="mb-3 text-sub text-tx-4">Cet ordinateur n'a pas encore de données : il reçoit celles de l'espace.</p>
      )}
      <p className="mb-3 text-sub leading-[1.5] text-tx-4">
        Le fournisseur de stockage et toute personne ayant accès à cet emplacement pourront lire les données synchronisées.
      </p>
      {error ? <p role="alert" className="text-sub text-st-c">{error}</p> : null}
    </ModalHost>
  );
}
