import { cn } from "@/shared/lib/cn";
import { Button, Icon, StatusPill } from "@/shared/ui";
import type { AiStatus } from "../../model/aiStatus";

/**
 * En-tête de l'écran IA : fournisseur actif, modèle, état, action de test.
 *
 * Un seul bloc, sans carte ni bordure lourde : l'information la plus utile —
 * « est-ce que ça marche ? » — se lit d'un coup d'œil.
 */
export function AiHero({
  logo,
  label,
  model,
  etat,
  testMessage,
  testLabel = "Tester la connexion",
  busy,
  testDisabled = false,
  onTest,
}: {
  /** Logo marque ; `null` → icône générique `smart_toy` (IA locale sans famille active). */
  logo: { src: string; mono: boolean } | null;
  label: string;
  model: string;
  etat: AiStatus;
  /** Message du dernier échec, affiché sous l'état ; `null` sinon. */
  testMessage: string | null;
  testLabel?: string;
  busy: boolean;
  testDisabled?: boolean;
  onTest: () => void;
}) {
  return (
    <section className="flex flex-wrap items-center gap-x-4 gap-y-3 rounded-card border border-line bg-surface px-[18px] py-4">
      <span className="relative flex size-12 flex-none items-center justify-center rounded-tile bg-fill">
        {logo ? (
          <img
            src={logo.src}
            alt=""
            width={28}
            height={28}
            className={cn("size-7", logo.mono && "dark:invert")}
          />
        ) : (
          <Icon name="smart_toy" size={28} className="text-ink-muted" />
        )}
      </span>
      <div className="min-w-[200px] flex-1">
        {/* Pas d'intitulé « Fournisseur » ici : la section juste en dessous porte déjà ce
            libellé, et le répéter ajoutait un niveau de titre pour rien. */}
        <p className="text-title text-ink">{label}</p>
        <p className="mt-0.5 truncate font-mono tabular text-note text-ink-faint">
          {model || "Aucun modèle"}
        </p>
      </div>
      <div className="flex flex-none items-center gap-2.5">
        <StatusPill tone={etat.tone}>{etat.label}</StatusPill>
        <Button icon="wifi" disabled={busy || testDisabled} onClick={onTest}>
          {busy ? "Test en cours…" : testLabel}
        </Button>
      </div>
      {testMessage ?? etat.hint ? (
        <p
          role="status"
          className={cn(
            "w-full text-note leading-relaxed",
            testMessage ? "text-danger" : "text-ink-faint",
          )}
        >
          {testMessage ?? etat.hint}
        </p>
      ) : null}
    </section>
  );
}
