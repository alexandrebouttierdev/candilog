import type { ReactNode } from "react";
import type { ResumeGeneration, GeneratedResume } from "@/features/ai/model/types";
import type { CoverLetter, CoverLetterExport } from "../../services/documentsService";
import { documentsService } from "../../services/documentsService";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { ContextBarAccessory, ContextNote, ContextSearch } from "@/app/layout/ContextBar";
import { FormField, Icon, TextInput } from "@/shared/ui";

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

export function AtsChip({ score }: { score: number }) {
  const tone = score >= 80 ? "bg-success-tint text-success" : score >= 65 ? "bg-warning-tint text-warning" : "bg-neutral-tint text-ink-muted";
  return <span className={`rounded-tag px-1.5 py-0.5 text-[10.5px] font-semibold ${tone}`}>ATS {score}</span>;
}

export function Champ({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return <FormField label={label}>{(props) => <TextInput {...props} value={value} onChange={(e) => onChange(e.target.value)} />}</FormField>;
}
export function message(error: unknown): string { return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite."; }
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
