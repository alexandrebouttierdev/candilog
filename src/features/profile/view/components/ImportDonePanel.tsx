import type { ImportProfileResult } from "@/shared/types/generated/profile";
import type { AiExecution } from "@/features/ai/model/types";
import { Icon, StatCard } from "@/shared/ui";
import { formatAiSummary, formatDuration } from "@/shared/lib/duration";

/** Bilan d'un import réussi : totaux en cartes, pas une liste à plat. */
export function ImportDonePanel({
  result,
  totalMs,
  aiMetrics,
}: {
  result: ImportProfileResult;
  totalMs: number;
  aiMetrics: Pick<AiExecution<unknown>, "elapsed_ms" | "tokens_used"> | null;
}) {
  return (
    <div className="pt-3">
      <div className="mb-5 flex items-start gap-3.5">
        <span className="flex size-[50px] flex-none items-center justify-center rounded-card bg-success-tint text-success">
          <Icon name="check_circle" size={26} />
        </span>
        <div className="min-w-0">
          <h3 className="text-heading text-ink">Profil importé</h3>
          <p className="mt-1 text-body text-ink-muted">{summarySentence(result)}</p>
        </div>
      </div>
      <div className="grid grid-cols-3 gap-2.5">
        <StatCard icon="playlist_add" tone="success" label="Ajoutés" value={String(result.added)} />
        <StatCard icon="sync" tone="accent" label="Mis à jour" value={String(result.replaced)} />
        <StatCard icon="block" tone="neutral" label="Ignorés" value={String(result.skipped)} />
      </div>
      <p className="mt-4 flex items-center gap-1.5 text-label text-ink-faint">
        <Icon name="schedule" size={15} className="flex-none" />
        Terminé en {formatDuration(totalMs)}
      </p>
      {aiMetrics ? (
        <p className="mt-1.5 flex items-center gap-1.5 text-label text-ink-faint">
          <Icon name="smart_toy" size={15} className="flex-none" />
          {formatAiSummary("Analysé", aiMetrics.elapsed_ms, aiMetrics.tokens_used)}
        </p>
      ) : null}
    </div>
  );
}

function summarySentence(result: ImportProfileResult): string {
  if (result.added > 0 && result.replaced === 0) {
    return `${result.added} élément${result.added > 1 ? "s" : ""} ont été ajoutés à votre profil.`;
  }
  if (result.replaced > 0 && result.added === 0) {
    return `${result.replaced} élément${result.replaced > 1 ? "s" : ""} existants ont été mis à jour.`;
  }
  if (result.added === 0 && result.replaced === 0) {
    return "Aucune donnée nouvelle n'a été enregistrée.";
  }
  return `${result.added} ajoutés · ${result.replaced} mis à jour · ${result.skipped} ignorés.`;
}
