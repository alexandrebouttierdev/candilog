import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, Icon, PageHeader, StatusPill } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { UpdateInfo } from "@/shared/types/generated/settings";
import { useUpdatesViewModel } from "../../viewmodel/useUpdatesViewModel";
import { SettingsBody, SettingsCard } from "../components/SettingsUi";

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
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="system_update"
        title="Mises à jour"
        subtitle="Candilog ne vérifie rien sans votre demande"
      />
      <SettingsBody>
        {/* Colonne bornée et blocs sur surface, comme l'écran Intelligence artificielle :
            l'écran ne porte que quatre faits, les étaler sur 1200 px les dilue. */}
        <div className="flex min-w-0 max-w-[760px] flex-col gap-4">
          <section className="min-w-0 overflow-hidden rounded-card border border-line bg-surface">
            <div className="flex flex-wrap items-start gap-x-4 gap-y-3 px-[18px] py-4">
              <Vignette update={vm.update} error={vm.error} installerOpened={vm.installerOpened} />
              <div className="min-w-[200px] flex-1">
                <Message
                  update={vm.update}
                  busy={vm.busy}
                  error={vm.error}
                  installerOpened={vm.installerOpened}
                />
              </div>
              <div className="flex flex-none flex-col items-end gap-2.5">
                <Etat update={vm.update} busy={vm.busy} installerOpened={vm.installerOpened} />
                {vm.update ? (
                  <Button
                    variant="primary"
                    icon="download"
                    disabled={enCours}
                    onClick={() => void vm.download()}
                  >
                    Mettre à jour
                  </Button>
                ) : (
                  <Button icon="refresh" disabled={enCours} onClick={() => void vm.check()}>
                    Rechercher une mise à jour
                  </Button>
                )}
              </div>
            </div>

            <div className="flex flex-wrap gap-x-10 gap-y-4 border-t border-line px-[18px] py-4">
              <div className="min-w-[150px] flex-1">
                <p className="text-eyebrow uppercase text-ink-label">Version installée</p>
                <p className="mt-1.5 font-mono tabular text-heading tracking-tight text-ink">
                  {vm.version}
                </p>
              </div>
              {vm.update ? (
                <div className="min-w-[150px] flex-1">
                  <p className="text-eyebrow uppercase text-ink-label">Nouvelle version</p>
                  <p className="mt-1.5 font-mono tabular text-heading tracking-tight text-accent-text">
                    {vm.update.version}
                  </p>
                </div>
              ) : null}
            </div>

            {vm.progress !== null && !vm.installerOpened ? (
              <div className="border-t border-line px-[18px] py-4" aria-label="Téléchargement">
                <div
                  role="progressbar"
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-valuenow={vm.progress}
                  aria-label="Téléchargement de la mise à jour"
                  className="h-1.5 overflow-hidden rounded-pill bg-neutral-tint"
                >
                  <div
                    className="h-full bg-accent transition-[width] duration-hover"
                    style={{ width: `${vm.progress}%` }}
                  />
                </div>
                <p className="mt-2 font-mono tabular text-meta text-ink-faint">
                  {vm.progress} % téléchargés
                </p>
              </div>
            ) : null}
          </section>

          {vm.update?.notes ? (
            <SettingsCard icon="new_releases" title="Nouveautés">
              <p className="text-body leading-relaxed whitespace-pre-line text-ink-muted">
                {vm.update.notes}
              </p>
            </SettingsCard>
          ) : null}
        </div>
      </SettingsBody>
    </div>
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
      ? { icon: "warning" as const, classes: "bg-danger-tint text-danger" }
      : installerOpened || update
        ? { icon: "new_releases" as const, classes: "bg-success-tint text-success" }
        : update === null
          ? { icon: "check_circle" as const, classes: "bg-success-tint text-success" }
          : { icon: "system_update" as const, classes: "bg-fill text-ink-muted" };

  return (
    <span
      aria-hidden
      className={cn(
        "flex size-11 flex-none items-center justify-center rounded-tile",
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
  if (busy === "check") return <StatusPill tone="neutral">Vérification…</StatusPill>;
  if (busy === "download") return <StatusPill tone="neutral">Téléchargement…</StatusPill>;
  if (installerOpened) return <StatusPill tone="success">Installeur ouvert</StatusPill>;
  if (update) {
    return (
      <StatusPill tone="success" icon="new_releases">
        Mise à jour disponible
      </StatusPill>
    );
  }
  if (update === null) return <StatusPill tone="success">À jour</StatusPill>;
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
      <p role="status" className="flex items-start gap-2 text-note leading-relaxed text-danger">
        <Icon name="warning" size={16} className="mt-px flex-none" />
        {error}
      </p>
    );
  }
  if (busy === "download") {
    return (
      <p className="text-note leading-relaxed text-ink-muted">
        Téléchargement en cours. L’installeur sera enregistré dans vos téléchargements ; son
        installation reste à votre main.
      </p>
    );
  }
  if (installerOpened) {
    return (
      <div>
        <p className="text-item font-semibold text-ink">Installeur ouvert</p>
        <p className="mt-1 text-note leading-relaxed text-ink-muted">
          Terminez l’installation dans la fenêtre système, puis redémarrez Candilog si elle le
          demande.
        </p>
      </div>
    );
  }
  if (update) {
    return (
      <div>
        <p className="text-item font-semibold text-ink">Une nouvelle version est disponible</p>
        <p className="mt-1 text-note leading-relaxed text-ink-muted">
          Le téléchargement reste à votre initiative, et l’installation à votre main.
        </p>
      </div>
    );
  }
  if (update === null) {
    return (
      <div>
        <p className="text-item font-semibold text-ink">Candilog est à jour</p>
        <p className="mt-1 text-note leading-relaxed text-ink-muted">
          Vous utilisez la dernière version disponible.
        </p>
      </div>
    );
  }
  return (
    <p className="text-note leading-relaxed text-ink-muted">
      Aucune vérification n’a encore été faite pendant cette session.
    </p>
  );
}
