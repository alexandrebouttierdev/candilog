import { Button } from "@/shared/ui";
import logoCandilog from "@/assets/logo-candilog.svg";
import { openExternal } from "@/shared/services/external-link";
import { useUiStore } from "@/shared/lib/ui-store";
import { useAboutViewModel } from "../../viewmodel/useAboutViewModel";
import { SettingsRow, SettingsSection } from "../components/SettingsSection";

/** Identité du produit : qui l'a fait, où vivent les données, comment mettre à jour. */
export function AboutPage() {
  const openSettings = useUiStore((state) => state.openSettings);
  const about = useAboutViewModel();

  return (
    <SettingsSection title="À propos" description="Candilog, un produit indépendant. Candidatures, réseau et documents — tout reste ici.">
      <div className="flex items-center gap-3.5 border-b border-bd-soft py-3">
        <span className="flex size-11 flex-none items-center justify-center rounded-r9 bg-group">
          <img src={logoCandilog} alt="" width={26} height={26} className="size-[26px]" />
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-row font-medium text-tx">Candilog</p>
          <p className="text-sub text-tx-5">Application de bureau</p>
        </div>
        <p className="flex-none font-mono text-caps text-tx-3">{about.version}</p>
      </div>
      <SettingsRow label="Données" hint="Conservées sur cet ordinateur, jamais sur un serveur Candilog.">
        <span className="text-small text-tx-4">Locales</span>
      </SettingsRow>
      <SettingsRow label="Intelligence artificielle" hint="Vous choisissez le fournisseur et le modèle de chaque tâche.">
        <span className="text-small text-tx-4">Sous votre contrôle</span>
      </SettingsRow>
      <SettingsRow label="Conçu et développé par" hint="Alexandre Bouttier">
        <span className="flex gap-1.5">
          <Button size="compact" onClick={() => void openExternal("https://www.alexandrebouttier.fr")}>
            Visiter le site
          </Button>
          <Button variant="primary" size="compact" onClick={() => openSettings("updates")}>
            Vérifier les mises à jour
          </Button>
        </span>
      </SettingsRow>
    </SettingsSection>
  );
}
