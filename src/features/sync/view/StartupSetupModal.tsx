import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PATHS } from "@/shared/lib/paths";
import { useUiStore } from "@/shared/lib/ui-store";
import { ModalHost } from "@/shared/ui";
import {
  readStartupSetupProgress,
  saveStartupSetupProgress,
  type StartupSetupProgress,
  type StartupStorageChoice,
} from "../model/startup-setup-progress";
import { useSyncStatus } from "../viewmodel/useSyncStatus";

/**
 * Demande uniquement les choix de configuration nécessaires au premier démarrage.
 * Le parcours n'est pas un tour guidé : l'écran Aujourd'hui reste ouvert en arrière-plan.
 */
export function StartupSetupModal() {
  const navigate = useNavigate();
  const settings = useUiStore((state) => state.settings);
  const openSettings = useUiStore((state) => state.openSettings);
  const { data: status, isPending, refetch } = useSyncStatus();
  const [storedProgress, setStoredProgress] = useState(readStartupSetupProgress);
  const progress =
    storedProgress ??
    (status && status.provider !== "local"
      ? { storage: "cloud", aiPromptDismissed: false }
      : null);
  const [choice, setChoice] = useState<StartupStorageChoice | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [cloudAttempted, setCloudAttempted] = useState(false);
  const settingsWereOpen = useRef(false);

  useEffect(() => {
    if (storedProgress === null && status && status.provider !== "local") {
      const next = { storage: "cloud", aiPromptDismissed: false } satisfies StartupSetupProgress;
      saveStartupSetupProgress(next);
    }
  }, [storedProgress, status]);

  useEffect(() => {
    if (settingsWereOpen.current && settings === null && cloudAttempted) {
      void refetch();
    }
    settingsWereOpen.current = settings !== null;
  }, [cloudAttempted, refetch, settings]);

  const isAiStep = progress !== null && !progress.aiPromptDismissed;
  const hasPendingStep = progress === null || !progress.aiPromptDismissed;
  const open =
    !dismissed &&
    settings === null &&
    hasPendingStep &&
    (progress !== null || !isPending);

  const saveProgress = (next: StartupSetupProgress) => {
    saveStartupSetupProgress(next);
    setStoredProgress(next);
    setDismissed(false);
  };

  const continueSetup = () => {
    if (isAiStep && progress) {
      saveProgress({ ...progress, aiPromptDismissed: true });
      void navigate(PATHS.ai);
      return;
    }
    if (choice === "local") {
      saveProgress({ storage: "local", aiPromptDismissed: false });
      return;
    }
    if (choice === "cloud") {
      setCloudAttempted(true);
      openSettings("data");
    }
  };

  const postponeAi = () => {
    if (progress) saveProgress({ ...progress, aiPromptDismissed: true });
  };

  return (
    <ModalHost
      open={open}
      title={isAiStep ? "Un assistant IA, quand vous le souhaitez" : "Bienvenue dans Candilog"}
      subtitle={
        isAiStep
          ? "Votre choix est enregistré. Vous pouvez configurer l’intelligence artificielle maintenant ou y revenir plus tard."
          : "Choisissez où conserver vos données. Vous pourrez modifier ce choix dans les réglages."
      }
      footer_note={isAiStep ? "ÉTAPE 2 SUR 2" : "ÉTAPE 1 SUR 2 · aucun envoi sans votre choix"}
      cancelLabel={isAiStep ? "Plus tard" : "Fermer"}
      submitLabel={isAiStep ? "Configurer l’IA maintenant" : choice === "cloud" ? "Configurer le cloud" : "Continuer"}
      submitDisabled={!isAiStep && choice === null}
      onClose={isAiStep ? postponeAi : () => setDismissed(true)}
      onSubmit={continueSetup}
      width="620px"
    >
      {isAiStep ? (
        <AiSetupStep />
      ) : (
        <StorageSetupStep
          choice={choice}
          cloudAttempted={cloudAttempted}
          onSelect={setChoice}
        />
      )}
    </ModalHost>
  );
}

function StorageSetupStep({
  choice,
  cloudAttempted,
  onSelect,
}: {
  choice: StartupStorageChoice | null;
  cloudAttempted: boolean;
  onSelect: (choice: StartupStorageChoice) => void;
}) {
  return (
    <div className="space-y-3.5">
      <fieldset className="grid gap-2.5 sm:grid-cols-2">
        <legend className="sr-only">Stockage des données</legend>
        <StorageOption
          value="local"
          selected={choice === "local"}
          title="Sur cet ordinateur"
          description="Vos données restent ici. Pas de compte ni de connexion nécessaires."
          note="Privé · disponible hors ligne"
          onSelect={onSelect}
        />
        <StorageOption
          value="cloud"
          selected={choice === "cloud"}
          title="Sur plusieurs appareils"
          description="Reliez un dossier, Google Drive ou OneDrive pour synchroniser vos données."
          note="Configuration à votre rythme"
          onSelect={onSelect}
        />
      </fieldset>
      <p className="rounded-r8 bg-group px-3 py-2.5 text-sub leading-[1.5] text-tx-5">
        Le stockage local est le choix par défaut. La synchronisation cloud n’est pas chiffrée :
        les données sont accessibles au fournisseur choisi.
      </p>
      {cloudAttempted ? (
        <p className="text-sub text-st-a" role="status">
          La configuration n’est pas terminée. Vos données restent sur cet ordinateur ; vous pouvez réessayer ou choisir le stockage local.
        </p>
      ) : null}
    </div>
  );
}

function StorageOption({
  value,
  selected,
  title,
  description,
  note,
  onSelect,
}: {
  value: StartupStorageChoice;
  selected: boolean;
  title: string;
  description: string;
  note: string;
  onSelect: (choice: StartupStorageChoice) => void;
}) {
  return (
    <label
      className={`group flex min-h-[148px] cursor-pointer items-start gap-2.5 rounded-r9 border p-3.5 transition-color ${
        selected
          ? "border-ac bg-ac-soft shadow-[inset_0_0_0_1px_var(--ac)]"
          : "border-bd-soft bg-surface hover:border-ac hover:bg-elev"
      }`}
    >
      <input
        type="radio"
        name="startup-storage"
        value={value}
        checked={selected}
        onChange={() => onSelect(value)}
        className="mt-0.5 size-4 flex-none accent-[var(--ac)]"
      />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-entry text-tx-2">{title}</span>
        <span className="mt-2.5 text-sub leading-[1.45] text-tx-4">{description}</span>
        <span className="mt-auto pt-3 font-mono text-mono-sm text-tx-6">{note}</span>
      </span>
    </label>
  );
}

function AiSetupStep() {
  return (
    <div className="rounded-r9 border border-bd-soft bg-surface px-4 py-4">
      <div className="flex items-start gap-3">
        <span
          aria-hidden="true"
          className="flex size-9 flex-none items-center justify-center rounded-r8 bg-ac-soft text-ui font-semibold text-ac-tx"
        >
          IA
        </span>
        <div>
          <h3 className="serif-title text-lead text-tx">Un coup de main, sans obligation</h3>
          <p className="mt-1.5 text-body leading-[1.55] text-tx-4">
            Configurez un fournisseur local ou en ligne pour vous aider à rédiger et analyser vos documents.
            Vous gardez le contrôle de chaque envoi.
          </p>
        </div>
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-bd-soft pt-3 text-sub text-tx-5">
        <span>Choix du fournisseur</span>
        <span aria-hidden="true">·</span>
        <span>Configuration réversible</span>
        <span aria-hidden="true">·</span>
        <span>Disponible plus tard dans Candilog</span>
      </div>
    </div>
  );
}
