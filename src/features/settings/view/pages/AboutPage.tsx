import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, InspectorRow, PageHeader } from "@/shared/ui";
import logoCandilog from "@/assets/logo-candilog.svg";
import { settingsService } from "../../services/settingsService";
import { A_ABOUT_KEY } from "../../viewmodel/useSettingsViewModel";
import { SettingsBody, SettingsCard } from "../components/SettingsUi";
import { useUiStore } from "@/shared/lib/ui-store";

/** Identité du produit : qui l'a fait, où vivent les données, comment mettre à jour. */
export function AboutPage() {
  const navigate = useNavigate();
  const setOnboarding = useUiStore((state) => state.setOnboarding);
  const info = useQuery({ queryKey: A_ABOUT_KEY, queryFn: settingsService.about });
  const version = info.data?.version ?? "…";

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader icon="info" title="À propos" subtitle="Candilog, un produit indépendant" />
      <SettingsBody>
        <div className="mx-auto flex w-full max-w-[720px] flex-col gap-4">
          <div className="flex items-start gap-4 py-1">
            <span className="flex size-11 flex-none items-center justify-center rounded-control bg-fill">
              <img src={logoCandilog} alt="" width={26} height={26} className="size-[26px]" />
            </span>
            <div className="min-w-0 flex-1">
              <p className="text-eyebrow uppercase text-ink-label">Application</p>
              <p className="mt-1 text-title text-ink">Candilog</p>
              <p className="mt-1 text-note text-ink-faint">
                Candidatures, réseau et documents — tout reste ici.
              </p>
            </div>
            <p className="tabular flex-none pt-5 text-item font-semibold text-ink">{version}</p>
          </div>

          <SettingsCard icon="inventory_2" title="Sur cet appareil">
            <InspectorRow label="Données">Conservées sur cet ordinateur</InspectorRow>
            <InspectorRow label="IA">Vous choisissez le fournisseur et le modèle</InspectorRow>
          </SettingsCard>

          <SettingsCard icon="tips_and_updates" title="Découvrir Candilog">
            <div className="flex flex-wrap items-center gap-3">
              <p className="min-w-0 flex-1 text-body leading-relaxed text-ink-muted">
                La visite guidée des écrans, affichée au premier lancement. La rejouer ne
                touche à aucune de vos données.
              </p>
              <Button
                variant="secondary"
                icon="tips_and_updates"
                onClick={() => setOnboarding(true)}
              >
                Revoir la présentation
              </Button>
            </div>
          </SettingsCard>

          <SettingsCard icon="badge" title="Conçu et développé par">
            <div className="flex flex-wrap items-center gap-3">
              <p className="min-w-0 flex-1 text-section text-ink">Alexandre Bouttier</p>
              <Button
                variant="secondary"
                icon="open_in_new"
                onClick={() => {
                  void import("@tauri-apps/plugin-opener").then(({ openUrl }) =>
                    openUrl("https://www.alexandrebouttier.fr"),
                  );
                }}
              >
                Visiter le site
              </Button>
              <Button
                variant="primary"
                icon="system_update"
                onClick={() => void navigate("/settings/updates")}
              >
                Vérifier les mises à jour
              </Button>
            </div>
          </SettingsCard>
        </div>
      </SettingsBody>
    </div>
  );
}
