import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useRegisterCommands } from "@/shared/lib/commands";
import type { Command } from "@/shared/lib/commands";
import { PATHS, applicationsPath } from "@/shared/lib/paths";
import { useUiStore } from "@/shared/lib/ui-store";
import { useAiIndicator } from "@/app/layout/useAiIndicator";
import { useNavCounts } from "@/app/layout/useNavCounts";

/**
 * Commandes de la coque, toujours présentes dans la palette : créer, aller à, réglages.
 * Les écrans y ajoutent leurs actions sur la sélection par `useRegisterCommands`.
 *
 * Seules les actions qui aboutissent réellement y figurent : une commande de la palette
 * qui n'ouvrirait rien serait pire qu'une commande absente.
 */
export function useShellCommands() {
  const navigate = useNavigate();
  const openSettings = useUiStore((state) => state.openSettings);
  const ai = useAiIndicator();
  const counts = useNavCounts();

  // Le modèle et la localité de la tâche sont imprimés sur chaque action IA : l'utilisateur
  // sait, avant de lancer, qui la fera et si ses données quittent la machine (`DECISIONS.md` D3).
  const resumeDetail = ai.taskDetail("generate_resume");
  const letterDetail = ai.taskDetail("write_letter");
  const analysisDetail = ai.taskDetail("analyze_resume");

  const commands = useMemo<readonly Command[]>(() => {
    const go = (path: string) => () => void navigate(path);
    const nombre = (value: number | undefined, suffixe: string) =>
      value === undefined ? undefined : `${value} ${suffixe}`;
    const list: Command[] = [
      { id: "create-application", group: "create", glyph: "+", label: "Nouvelle candidature", run: go(applicationsPath({ create: true })) },
      { id: "generate-resume", group: "create", glyph: "✦", label: "Générer un CV ciblé", detail: resumeDetail, keywords: "ia cv", run: go(PATHS.generateResume) },
      { id: "write-letter", group: "create", glyph: "✦", label: "Rédiger une lettre de motivation", detail: letterDetail, keywords: "ia lettre", run: go(PATHS.writeLetter) },
      { id: "analyze-resume", group: "create", glyph: "◫", label: "Analyser un CV face à une offre", detail: analysisDetail, keywords: "ats score", run: go(PATHS.analyzeResume) },
      { id: "go-today", group: "goto", glyph: "◷", label: "Aujourd'hui", detail: nombre(counts.today, "échéances"), shortcut: "g a", run: go(PATHS.today) },
      { id: "go-applications", group: "goto", glyph: "▤", label: "Candidatures", detail: nombre(counts.applications, "suivies"), shortcut: "g c", run: go(PATHS.applications) },
      { id: "go-kanban", group: "goto", glyph: "▤", label: "Candidatures — Kanban", run: go(PATHS.applicationsKanban) },
      { id: "go-calendar", group: "goto", glyph: "◷", label: "Candidatures — Calendrier", keywords: "entretiens relances", run: go(PATHS.calendar) },
      { id: "go-analytics", group: "goto", glyph: "▤", label: "Candidatures — Analyse", keywords: "statistiques", run: go(PATHS.analytics) },
      { id: "go-companies", group: "goto", glyph: "▤", label: "Relations — Entreprises", shortcut: "g r", run: go(PATHS.companies) },
      { id: "go-contacts", group: "goto", glyph: "▤", label: "Relations — Contacts", keywords: "réseau", run: go(PATHS.contacts) },
      { id: "go-documents", group: "goto", glyph: "▤", label: "Documents — CV", shortcut: "g d", run: go(PATHS.documents) },
      { id: "go-letters", group: "goto", glyph: "▤", label: "Documents — Lettres", run: go(PATHS.letters) },
      { id: "go-ai", group: "goto", glyph: "✦", label: "Intelligence artificielle", keywords: "fournisseur modèle", run: go(PATHS.ai) },
      { id: "go-profile", group: "goto", glyph: "▤", label: "Profil", detail: counts.profile === undefined ? undefined : `${counts.profile} % complété`, shortcut: "g p", run: go(PATHS.profile) },
      { id: "settings", group: "settings", glyph: "◫", label: "Réglages", shortcut: "mod+,", run: () => openSettings() },
      { id: "settings-appearance", group: "settings", glyph: "◫", label: "Apparence", keywords: "thème densité animations", run: () => openSettings("appearance") },
      { id: "settings-data", group: "settings", glyph: "◫", label: "Données", keywords: "sauvegarde restauration", run: () => openSettings("data") },
      { id: "settings-shortcuts", group: "settings", glyph: "◫", label: "Raccourcis clavier", run: () => openSettings("shortcuts") },
      { id: "settings-updates", group: "settings", glyph: "↻", label: "Mises à jour", run: () => openSettings("updates") },
      { id: "settings-about", group: "settings", glyph: "◫", label: "À propos", run: () => openSettings("about") },
    ];
    return list;
  }, [navigate, openSettings, resumeDetail, letterDetail, analysisDetail, counts.today, counts.applications, counts.profile]);

  useRegisterCommands(commands);
}
