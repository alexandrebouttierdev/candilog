import { Bar, BarChart, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { Step } from "@/shared/types/generated/analytics";
import { EmptyState } from "@/shared/ui";
import { CHART_INK } from "./chartTheme";
import { ChartTooltip } from "./ChartTooltip";

/** Teintes des quatre statuts, dans l'ordre où le backend renvoie les étapes. */
const COLORS = [CHART_INK.neutral, CHART_INK.warning, CHART_INK.success, CHART_INK.danger];

/**
 * Répartition du pipeline : une bande empilée à 100 %, puis sa légende chiffrée.
 *
 * Bande plutôt que camembert : quatre parts d'un même tout se comparent mieux sur un axe
 * commun, et la bande occupe la largeur disponible d'une carte dense.
 *
 * La palette est celle des statuts du design system, où le vert et le rouge sont proches
 * pour une deutéranopie. C'est pourquoi la légende nomme et chiffre systématiquement chaque
 * segment, et qu'un filet de deux pixels les sépare : la couleur ne porte jamais seule
 * l'information.
 */
export function PipelineChart({ steps }: { steps: readonly Step[] }) {
  const total = steps.reduce((sum, step) => sum + step.count, 0);

  if (total === 0) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Pipeline vide"
        description="Les statuts de vos candidatures formeront cette répartition."
      />
    );
  }

  // Une seule catégorie porte toutes les étapes : c'est ce qui les empile sur une bande.
  const stackedData = [
    Object.fromEntries(steps.map((step) => [step.label, step.count])) as Record<string, number>,
  ];

  return (
    <div>
      <div style={{ height: 14 }} role="img" aria-label="Répartition des candidatures par statut">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart
            data={stackedData}
            layout="vertical"
            margin={{ top: 0, right: 0, bottom: 0, left: 0 }}
            accessibilityLayer
          >
            <XAxis type="number" domain={[0, total]} hide />
            <YAxis type="category" hide />
            {steps.map((step, index) => (
              <Bar
                key={step.label}
                dataKey={step.label}
                stackId="pipeline"
                isAnimationActive={false}
                radius={2}
              >
                <Cell fill={COLORS[index] ?? CHART_INK.neutral} stroke={CHART_INK.gap} strokeWidth={2} />
              </Bar>
            ))}
            <Tooltip
              shared={false}
              cursor={false}
              content={({ active, payload }) => {
                const part = active ? payload?.[0] : undefined;
                if (!part) return null;
                const count = typeof part.value === "number" ? part.value : 0;
                return (
                  <ChartTooltip title={String(part.name)}>
                    {`${count} candidature${count > 1 ? "s" : ""}`}
                  </ChartTooltip>
                );
              }}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>

      <ul className="mt-3.5 flex flex-wrap gap-x-[22px] gap-y-2">
        {steps.map((step, index) => (
          <li key={step.label} className="flex items-center gap-[7px]">
            <span
              aria-hidden
              style={{ backgroundColor: COLORS[index] ?? CHART_INK.neutral }}
              className="size-1.5 rounded-full"
            />
            <span className="text-note text-ink-muted">{step.label}</span>
            <span className="tabular text-note font-semibold text-ink">{step.count}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
