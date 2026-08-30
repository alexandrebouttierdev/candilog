import type { ReactNode } from "react";
import type { ResumeGeneration, GeneratedResume } from "@/features/ai/model/types";
import type { CoverLetter, CoverLetterExport } from "../../services/documentsService";
import { documentsService } from "../../services/documentsService";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { ContextBarAccessory, ContextNote, ContextSearch } from "@/app/layout/ContextBar";
import { Button, FormField, Icon, TextArea, TextInput } from "@/shared/ui";
import { useUiStore } from "@/shared/lib/ui-store";

export const RESUME_KEY = ["documents", "cv"] as const;
export const COVER_LETTERS_KEY = ["documents", "lettres"] as const;

export function Screen({
  header,
  children,
  padded = true,
  search,
}: {
  header: ReactNode;
  children: ReactNode;
  padded?: boolean;
  search?: { value: string; onChange: (value: string) => void; placeholder: string };
}) {
  return (
    <div className="flex h-full flex-col">
      {search ? (
        <ContextBarAccessory>
          <ContextSearch
            value={search.value}
            placeholder={search.placeholder}
            onChange={search.onChange}
            width={230}
          />
        </ContextBarAccessory>
      ) : (
        <ContextBarAccessory>
          <ContextNote>Documents locaux · génération IA</ContextNote>
        </ContextBarAccessory>
      )}
      {header}
      <div className={padded ? "min-h-0 flex-1 overflow-y-auto p-5 min-[1200px]:p-6" : "flex min-h-0 flex-1 flex-col overflow-hidden"}>
        {children}
      </div>
    </div>
  );
}

export function HeaderBadge({ children, icon = "auto_awesome" }: { children: ReactNode; icon?: string }) {
  return (
    <span className="inline-flex items-center gap-[5px] rounded-pill bg-accent-tint px-2.5 py-[5px] text-label font-mid text-accent">
      <Icon name={icon} size={15} />
      {children}
    </span>
  );
}

/**
 * Mention accompagnant le texte libre renvoyé par le modèle.
 *
 * Le score, lui, est calculé par Candilog. Le récapitulatif et les suggestions traversent
 * en revanche la génération sans recadrage sur les faits du profil : l'offre analysée étant
 * un contenu externe, ce texte se lit comme un commentaire, pas comme un résultat vérifié.
 */
export function TexteNonVerifie() {
  return (
    <p className="flex items-start gap-1.5 text-meta text-ink-faint">
      <Icon name="info" size={14} className="mt-px flex-none" />
      Commentaire rédigé par le modèle, à partir de l’offre fournie. Le score, lui, est
      calculé par Candilog.
    </p>
  );
}

export function AtsChip({ score }: { score: number }) {
  const tone = score >= 80 ? "bg-success-tint text-success" : score >= 65 ? "bg-warning-tint text-warning" : "bg-neutral-tint text-ink-muted";
  return <span className={`rounded-tag px-1.5 py-0.5 text-[10.5px] font-semibold ${tone}`}>ATS {score}</span>;
}

/**
 * Champ d'offre ou de contexte, avec collage direct depuis le presse-papiers.
 *
 * Ces champs reçoivent une annonce entière copiée depuis un navigateur : le bouton évite
 * d'avoir à viser la zone de texte avant de faire Ctrl+V, et remplace le contenu existant
 * comme le ferait un collage sur une sélection complète.
 */
export function ChampOffre({
  label,
  value,
  rows,
  required = false,
  help,
  placeholder,
  onChange,
}: {
  label: string;
  value: string;
  rows: number;
  required?: boolean;
  help?: string;
  placeholder?: string;
  onChange: (value: string) => void;
}) {
  const notify = useUiStore((state) => state.notify);

  const coller = async () => {
    try {
      const texte = await navigator.clipboard.readText();
      if (!texte.trim()) {
        notify({ tone: "info", title: "Le presse-papiers est vide" });
        return;
      }
      onChange(texte);
    } catch {
      notify({
        tone: "error",
        title: "Collage impossible",
        detail: "Le presse-papiers n'est pas accessible ; utilisez Ctrl+V dans le champ.",
      });
    }
  };

  return (
    <FormField label={label} required={required} help={help}>
      {(props) => (
        <div className="flex flex-col gap-1.5">
          <TextArea
            {...props}
            rows={rows}
            value={value}
            placeholder={placeholder}
            onChange={(event) => onChange(event.target.value)}
          />
          <Button
            variant="ghost"
            size="dialog"
            icon="content_paste"
            className="self-end"
            onClick={() => void coller()}
          >
            Coller
          </Button>
        </div>
      )}
    </FormField>
  );
}

export function Champ({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return <FormField label={label}>{(props) => <TextInput {...props} value={value} onChange={(e) => onChange(e.target.value)} />}</FormField>;
}
export function message(error: unknown): string { return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite."; }
/** Détail d'un toast d'erreur : le message backend quand il y en a un, rien sinon. */
export function detail(error: unknown): string | undefined { return error instanceof AppError ? error.message : undefined; }
export function date(value: string): string { const d = new Date(value); return Number.isNaN(d.getTime()) ? value : new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "short", year: "numeric" }).format(d); }
export function isGeneration(value: unknown): value is ResumeGeneration { return typeof value === "object" && value !== null && "resume" in value && "analysis" in value; }
export function generationFromNavigation(state: unknown): { result: ResumeGeneration | null; name: string } {
  if (typeof state !== "object" || state === null || !("generation" in state)) {
    return { result: null, name: "" };
  }
  const payload = state as { generation?: ResumeGeneration; name?: string };
  if (!payload.generation) return { result: null, name: "" };
  return {
    result: payload.generation,
    name: payload.name ?? `CV — ${payload.generation.job_offer.title || "Version ciblée"}`,
  };
}
export function coverLetterFromNavigation(state: unknown): CoverLetter | null {
  if (typeof state !== "object" || state === null || !("cover_letter" in state)) return null;
  const payload = state as { cover_letter?: CoverLetter };
  return payload.cover_letter ?? null;
}
export function labelTone(tone: string): string {
  if (tone === "casual") return "Naturel";
  if (tone === "creative") return "Créatif";
  return "Formel";
}

export async function exportPdf(
  resume: GeneratedResume,
  notify: (toast: Omit<ToastMessage, "id">) => void,
) {
  try {
    const exported = await documentsService.exportPdf(resume);
    if (!exported) return;
    notify({ tone: "success", title: "CV exporté" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}

export async function exportLetterPdf(
  cover_letter: CoverLetterExport,
  notify: (toast: Omit<ToastMessage, "id">) => void,
) {
  try {
    const exported = await documentsService.exportCoverLetterPdf(cover_letter);
    if (!exported) return;
    notify({ tone: "success", title: "Lettre exportée" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}
