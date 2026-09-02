import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import type { UpdateInfo, UpdateProgress } from "@/shared/types/generated/settings";
import { Button, PageHeader, StatusPill } from "@/shared/ui";
import { settingsService } from "../../services/settingsService";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

const Version = settingsService.about;

/** Recherche et installation assistée des mises à jour. */
export function UpdatesPage() {
  const [version, setVersion] = useState("…");
  const [update, setUpdate] = useState<UpdateInfo | null | undefined>(undefined);
  const [busy, setBusy] = useState<"check" | "download" | null>(null);
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void Version().then((info) => setVersion(info.version)).catch(() => setVersion("inconnue"));
  }, []);

  useEffect(() => {
    let cancelled = false;
    let dispose: (() => void) | undefined;
    void listen<UpdateProgress>("update-progress", (event) => {
      setProgress(event.payload.progress);
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
          return;
        }
        dispose = unlisten;
      })
      .catch(() => {
        /* revue navigateur sans runtime Tauri */
      });
    return () => {
      cancelled = true;
      dispose?.();
    };
  }, []);

  const check = async () => {
    setBusy("check");
    setError(null);
    try {
      setUpdate(await settingsService.checkUpdate());
    } catch (error) {
      setError(error instanceof AppError ? error.message : "Vérification impossible.");
    } finally {
      setBusy(null);
    }
  };

  const download = async () => {
    if (!update?.asset) {
      const fallback = "https://github.com/alexandrebouttierdev/candilog/releases/latest";
      const page = update?.page_url ?? fallback;
      await openUrl(pageOfficielle(page) ? page : fallback);
      return;
    }
    setBusy("download");
    setError(null);
    setProgress(0);
    try {
      await settingsService.downloadUpdate();
    } catch (error) {
      setError(error instanceof AppError ? error.message : "Téléchargement impossible.");
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="system_update"
        title="Mises à jour"
        subtitle={`Version actuelle ${version}`}
      />
      <SettingsBody>
        <SettingsHero
          kicker="Version actuelle"
          title={version}
          description="Candilog vérifie les nouvelles versions uniquement lorsque vous le demandez."
        />
        <div className="grid gap-4 md:grid-cols-2">
          <ActionCard
            icon="refresh"
            title="Disponibilité"
            description="Interrogez la source officielle et comparez-la à votre version installée."
          >
            <div className="space-y-3">
              {update ? (
                <StatusPill tone="success" icon="new_releases">
                  {`Version ${update.version} disponible`}
                </StatusPill>
              ) : update === null ? (
                <StatusPill tone="neutral">Aucune mise à jour en attente</StatusPill>
              ) : null}
              {update ? (
                <Button variant="primary" icon="download" disabled={busy !== null} onClick={() => void download()}>
                  Télécharger la mise à jour
                </Button>
              ) : (
                <Button variant="primary" icon="refresh" disabled={busy !== null} onClick={() => void check()}>
                  Rechercher maintenant
                </Button>
              )}
              {error ? <p className="text-meta text-danger">{error}</p> : null}
            </div>
          </ActionCard>
          <ActionCard
            icon="verified"
            title="Installation maîtrisée"
            description="L’installeur adapté à votre système est téléchargé, son empreinte SHA-256 comparée à celle publiée avec la release, et il n’est enregistré qu’ensuite."
          >
            <p className="text-meta text-ink-muted">Aucune installation silencieuse</p>
            <p className="mt-1 text-meta text-ink-faint">
              L’empreinte atteste que le fichier est arrivé intact, et l’attestation de
              provenance publiée avec la release qu’il vient bien du dépôt officiel. Ni
              l’une ni l’autre n’est une signature de code : votre système peut afficher un
              avertissement « éditeur inconnu » à l’installation.
            </p>
          </ActionCard>
        </div>
        {progress !== null ? (
          <SettingsCard icon="download" title="Téléchargement">
            <div
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={progress}
              aria-label="Téléchargement de la mise à jour"
              className="h-2 overflow-hidden rounded-pill bg-neutral-tint"
            >
              <div className="h-full bg-accent" style={{ width: `${progress}%` }} />
            </div>
            <p className="mt-2 text-meta text-ink-muted">{progress} %</p>
          </SettingsCard>
        ) : null}
      </SettingsBody>
    </div>
  );
}

function pageOfficielle(url: string): boolean {
  try {
    const parsed = new URL(url);
    return (
      parsed.protocol === "https:" &&
      parsed.hostname === "github.com" &&
      parsed.pathname.startsWith("/alexandrebouttierdev/candilog/")
    );
  } catch {
    return false;
  }
}
