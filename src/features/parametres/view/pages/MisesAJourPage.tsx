import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AppError } from "@/shared/types/app-error";
import type { MiseAJour, ProgressionMaj } from "@/shared/types/generated/parametres";
import { Button, PageHeader, StatusPill } from "@/shared/ui";
import { parametresService } from "../../services/parametres.service";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

const VERSION = parametresService.aPropos;

/** Recherche et installation assistée des mises à jour. */
export function MisesAJourPage() {
  const [version, setVersion] = useState("…");
  const [maj, setMaj] = useState<MiseAJour | null | undefined>(undefined);
  const [busy, setBusy] = useState<"check" | "download" | null>(null);
  const [progression, setProgression] = useState<number | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);

  useEffect(() => {
    void VERSION().then((info) => setVersion(info.version)).catch(() => setVersion("inconnue"));
  }, []);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    void listen<ProgressionMaj>("maj-progression", (event) => {
      setProgression(event.payload.progression);
    })
      .then((unlisten) => {
        dispose = unlisten;
      })
      .catch(() => {
        /* revue navigateur sans runtime Tauri */
      });
    return () => dispose?.();
  }, []);

  const verifier = async () => {
    setBusy("check");
    setErreur(null);
    try {
      setMaj(await parametresService.verifierMaj());
    } catch (error) {
      setErreur(error instanceof AppError ? error.message : "Vérification impossible.");
    } finally {
      setBusy(null);
    }
  };

  const telecharger = async () => {
    if (!maj?.asset) {
      await openUrl(maj?.pageUrl ?? "https://github.com/alexandrebouttierdev/candilog-releases/releases/latest");
      return;
    }
    setBusy("download");
    setErreur(null);
    setProgression(0);
    try {
      await parametresService.telechargerMaj(maj.asset.url, maj.asset.nom);
    } catch (error) {
      setErreur(error instanceof AppError ? error.message : "Téléchargement impossible.");
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="flex h-full flex-col">
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
              {maj ? (
                <StatusPill tone="success" icon="new_releases">
                  {`Version ${maj.version} disponible`}
                </StatusPill>
              ) : maj === null ? (
                <StatusPill tone="neutral">Aucune mise à jour en attente</StatusPill>
              ) : null}
              {maj ? (
                <Button variant="primary" icon="download" disabled={busy !== null} onClick={() => void telecharger()}>
                  Télécharger la mise à jour
                </Button>
              ) : (
                <Button variant="primary" icon="refresh" disabled={busy !== null} onClick={() => void verifier()}>
                  Rechercher maintenant
                </Button>
              )}
              {erreur ? <p className="text-meta text-danger">{erreur}</p> : null}
            </div>
          </ActionCard>
          <ActionCard
            icon="verified"
            title="Installation maîtrisée"
            description="L’installeur adapté à votre système est téléchargé puis lancé avec le programme d’installation par défaut."
          >
            <p className="text-meta text-ink-muted">Aucune installation silencieuse</p>
          </ActionCard>
        </div>
        {progression !== null ? (
          <SettingsCard icon="download" title="Téléchargement">
            <div
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={progression}
              aria-label="Téléchargement de la mise à jour"
              className="h-2 overflow-hidden rounded-pill bg-neutral-tint"
            >
              <div className="h-full bg-accent" style={{ width: `${progression}%` }} />
            </div>
            <p className="mt-2 text-meta text-ink-muted">{progression} %</p>
          </SettingsCard>
        ) : null}
      </SettingsBody>
    </div>
  );
}
