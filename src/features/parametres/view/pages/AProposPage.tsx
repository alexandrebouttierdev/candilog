import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, Icon, PageHeader, StatusPill } from "@/shared/ui";
import { parametresService } from "../../services/parametres.service";
import { A_PROPOS_KEY } from "../../viewmodel/useParametresViewModel";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

/** Identité du produit, indépendance et données locales. */
export function AProposPage() {
  const navigate = useNavigate();
  const info = useQuery({ queryKey: A_PROPOS_KEY, queryFn: parametresService.aPropos });
  const version = info.data?.version ?? "…";

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader icon="info" title="À propos" subtitle="Candilog, un produit indépendant" />
      <SettingsBody>
        <SettingsHero
          kicker="Candilog desktop"
          title="Votre recherche d'emploi, enfin au même endroit."
          description="Un cockpit pour suivre vos candidatures, développer votre réseau et produire des documents professionnels cohérents."
        />
        <div className="flex flex-wrap items-center gap-2">
          <StatusPill tone="neutral">{`Version ${version}`}</StatusPill>
          <StatusPill tone="accent">Tauri · React · SQLite</StatusPill>
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          <ActionCard
            icon="lock"
            title="Vos données restent locales"
            description="Candidatures, contacts et documents sont conservés dans votre base SQLite."
          >
            <StatusPill tone="success">Local-first</StatusPill>
          </ActionCard>
          <ActionCard
            icon="desktop_windows"
            title="Une expérience vraiment native"
            description="Fenêtre desktop, navigation clavier et intégration au système, sans compte obligatoire."
          >
            <StatusPill tone="accent">100 % natif</StatusPill>
          </ActionCard>
          <ActionCard
            icon="smart_toy"
            title="Une IA sous votre contrôle"
            description="Vous choisissez le fournisseur, le modèle et les contenus à analyser."
          >
            <StatusPill tone="neutral">Configurable</StatusPill>
          </ActionCard>
        </div>
        <SettingsCard icon="person" title="Un produit indépendant">
          <div className="flex flex-wrap items-center gap-3">
            <div className="min-w-0 flex-1">
              <p className="text-meta font-semibold tracking-wide text-accent uppercase">Conçu et développé par</p>
              <p className="mt-1 flex items-center gap-2 text-section text-ink">
                <Icon name="badge" size={18} className="text-accent" />
                Alexandre Bouttier
              </p>
              <p className="mt-1 text-meta text-ink-muted">
                Pensé pour une recherche d'emploi exigeante, concrète et locale.
              </p>
            </div>
            <Button variant="secondary" icon="open_in_new" onClick={() => void openUrl("https://www.alexandrebouttier.fr")}>
              Visiter le site
            </Button>
            <Button variant="primary" icon="system_update" onClick={() => void navigate("/reglages/mises-a-jour")}>
              Vérifier les mises à jour
            </Button>
          </div>
        </SettingsCard>
      </SettingsBody>
    </div>
  );
}
