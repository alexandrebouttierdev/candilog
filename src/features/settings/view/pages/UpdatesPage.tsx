import { Button, Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { UpdateInfo } from "@/shared/types/generated/settings";
import { useUpdatesViewModel } from "../../viewmodel/useUpdatesViewModel";
import { SettingsRow, SettingsSection } from "../components/SettingsSection";

/**
 * Mises à jour : version installée, disponibilité, action.
 *
 * L'écran répond à trois questions et rien de plus — quelle version j'utilise, en existe-t-il
 * une plus récente, que dois-je faire. Le détail du mécanisme de téléchargement, qui n'aide
 * en rien à décider, a été retiré.
 */
export function UpdatesPage() {
  const vm = useUpdatesViewModel();
  const enCours = vm.busy !== null;

  return (
    <SettingsSection
      title="Mises à jour"
      description="Candilog ne vérifie rien sans votre demande ; le téléchargement et l’installation restent à votre main."
    >
      <div className="flex items-start gap-3.5 border-b border-bd-soft py-3.5">
        <Vignette update={vm.update} error={vm.error} installerOpened={vm.installerOpened} />
        <div className="min-w-0 flex-1">
          <Message update={vm.update} busy={vm.busy} error={vm.error} installerOpened={vm.installerOpened} />
        </div>
        <Etat update={vm.update} busy={vm.busy} installerOpened={vm.installerOpened} />
      </div>

      <SettingsRow label="Version installée">
        <span className="font-mono text-caps text-tx-3">{vm.version}</span>
      </SettingsRow>
      {vm.update ? (
        <SettingsRow label="Nouvelle version">
          <span className="font-mono text-caps text-ac-tx">{vm.update.version}</span>
        </SettingsRow>
      ) : null}
      <SettingsRow
        label={vm.update ? "Installer la nouvelle version" : "Vérifier maintenant"}
        hint={vm.update ? "L’installeur est enregistré dans vos téléchargements." : "Interroge la page des versions publiées."}
      >
        {vm.update ? (
          <Button variant="primary" size="compact" disabled={enCours} onClick={() => void vm.download()}>
            Mettre à jour
          </Button>
        ) : (
          <Button size="compact" disabled={enCours} onClick={() => void vm.check()}>
            Rechercher une mise à jour
          </Button>
        )}
      </SettingsRow>

      {vm.progress !== null && !vm.installerOpened ? (
        <div className="border-b border-bd-soft py-3.5" aria-label="Téléchargement">
          <div
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={vm.progress}
            aria-label="Téléchargement de la mise à jour"
            className="h-1 overflow-hidden rounded-r2 bg-chip"
          >
            <div className="h-full bg-ac transition-[width] duration-150" style={{ width: `${vm.progress}%` }} />
          </div>
          <p className="mt-2 font-mono text-caps text-tx-5">{vm.progress} % téléchargés</p>
        </div>
      ) : null}

      {vm.update?.notes ? (
        <section aria-label="Nouveautés" className="pt-4">
          <h3 className="caps mb-2">Nouveautés</h3>
          <p className="text-small leading-relaxed whitespace-pre-line text-tx-3">{vm.update.notes}</p>
        </section>
      ) : null}
    </SettingsSection>
  );
}

/**
 * Vignette d'état : elle reprend le sens de la pastille, en plus gros et sans mot.
 *
 * Un rectangle neutre aurait décoré sans informer ; ici la couleur et l'icône disent déjà
 * si l'on doit agir, avant même de lire la phrase.
 */
function Vignette({
  update,
  error,
  installerOpened,
}: {
  update: UpdateInfo | null | undefined;
  error: string | null;
  installerOpened: boolean;
}) {
  const apparence =
    error !== null
      ? { icon: "warning" as const, classes: "bg-tint-c-bg text-tint-c-tx" }
      : installerOpened || update
        ? { icon: "new_releases" as const, classes: "bg-tint-g-bg text-tint-g-tx" }
        : update === null
          ? { icon: "check_circle" as const, classes: "bg-tint-g-bg text-tint-g-tx" }
          : { icon: "system_update" as const, classes: "bg-group text-tx-4" };

  return (
    <span
      aria-hidden
      className={cn(
        "flex size-10 flex-none items-center justify-center rounded-r9",
        apparence.classes,
      )}
    >
      <Icon name={apparence.icon} size={22} />
    </span>
  );
}

/** Pastille d'état : vérification, à jour, mise à jour disponible, ou rien encore. */
function Etat({
  update,
  busy,
  installerOpened,
}: {
  update: UpdateInfo | null | undefined;
  busy: "check" | "download" | null;
  installerOpened: boolean;
}) {
  const chip = (text: string, tone: "neutral" | "good") => (
    <span
      className={cn(
        "inline-flex h-[22px] flex-none items-center rounded-r6 px-2 text-small",
        tone === "good" ? "bg-tint-g-bg text-tint-g-tx" : "bg-chip text-tx-3",
      )}
    >
      {text}
    </span>
  );
  if (busy === "check") return chip("Vérification…", "neutral");
  if (busy === "download") return chip("Téléchargement…", "neutral");
  if (installerOpened) return chip("Installeur ouvert", "good");
  if (update) return chip("Mise à jour disponible", "good");
  if (update === null) return chip("À jour", "good");
  return null;
}

/** Phrase qui explique l'état courant, ou l'échec de la dernière tentative. */
function Message({
  update,
  busy,
  error,
  installerOpened,
}: {
  update: UpdateInfo | null | undefined;
  busy: "check" | "download" | null;
  error: string | null;
  installerOpened: boolean;
}) {
  if (error !== null) {
    return (
      <p role="status" className="flex items-start gap-2 text-small leading-relaxed text-tint-c-tx">
        <Icon name="warning" size={16} className="mt-px flex-none" />
        {error}
      </p>
    );
  }
  if (busy === "download") {
    return (
      <p className="text-small leading-relaxed text-tx-4">
        Téléchargement en cours. L’installeur sera enregistré dans vos téléchargements ; son
        installation reste à votre main.
      </p>
    );
  }
  if (installerOpened) {
    return (
      <div>
        <p className="text-row font-medium text-tx">Installeur ouvert</p>
        <p className="mt-1 text-small leading-relaxed text-tx-4">
          Terminez l’installation dans la fenêtre système, puis redémarrez Candilog si elle le
          demande.
        </p>
      </div>
    );
  }
  if (update) {
    return (
      <div>
        <p className="text-row font-medium text-tx">Une nouvelle version est disponible</p>
        <p className="mt-1 text-small leading-relaxed text-tx-4">
          Le téléchargement reste à votre initiative, et l’installation à votre main.
        </p>
      </div>
    );
  }
  if (update === null) {
    return (
      <div>
        <p className="text-row font-medium text-tx">Candilog est à jour</p>
        <p className="mt-1 text-small leading-relaxed text-tx-4">
          Vous utilisez la dernière version disponible.
        </p>
      </div>
    );
  }
  return (
    <p className="text-small leading-relaxed text-tx-4">
      Aucune vérification n’a encore été faite pendant cette session.
    </p>
  );
}
