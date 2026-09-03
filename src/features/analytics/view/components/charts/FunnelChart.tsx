import {
  Bar,
  BarChart,
  Cell,
  LabelList,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { Step } from "@/shared/types/generated/analytics";
import { EmptyState } from "@/shared/ui";
import { AXIS_TICK, CHART_INK } from "./chartTheme";
import { ChartTooltip } from "./ChartTooltip";

/** Hauteur d'une étape, filet compris : quatre étapes tiennent dans la carte des maquettes. */
const HAUTEUR_ETAPE = 42;

/**
 * Conversion étape par étape : envoyées, réponses, entretiens, refus.
 *
 * Barres horizontales et non un entonnoir géométrique : les étapes ne s'emboîtent pas
 * réellement — un refus n'est pas un sous-ensemble des entretiens —, et une silhouette en
 * entonnoir laisserait croire le contraire. Chaque barre porte son libellé et sa valeur,
 * l'identité ne repose donc jamais sur la seule couleur.
 */
export function FunnelChart({ steps }: { steps: readonly Step[] }) {
  if (steps.every((step) => step.count === 0)) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Entonnoir vide"
        description="Il se construira dès la première candidature."
      />
    );
  }

  const data = steps.map((step) => ({ ...step }));
  const maximum = Math.max(...data.map((step) => step.count), 1);

  return (
    <>
      <div
        style={{ height: data.length * HAUTEUR_ETAPE }}
        role="img"
        aria-label="Conversion des candidatures, étape par étape"
      >
        <ResponsiveContainer width="100%" height="100%">
          <BarChart
            data={data}
            layout="vertical"
            margin={{ top: 0, right: 44, bottom: 0, left: 0 }}
            barCategoryGap="34%"
            accessibilityLayer
          >
            <XAxis type="number" domain={[0, maximum]} hide />
            <YAxis
              type="category"
              dataKey="label"
              width={104}
              tick={AXIS_TICK}
              tickLine={false}
              axisLine={false}
            />
            <Tooltip
              cursor={{ fill: CHART_INK.empty }}
              content={({ active, payload }) => {
                const etape = active ? (payload?.[0]?.payload as Step | undefined) : undefined;
                if (!etape) return null;
                return (
                  <ChartTooltip title={etape.label}>
                    {`${etape.count} candidature${etape.count > 1 ? "s" : ""} · ${etape.percentage} %`}
                  </ChartTooltip>
                );
              }}
            />
            <Bar dataKey="count" radius={[0, 4, 4, 0]} barSize={10} isAnimationActive={false}>
              {data.map((step) => (
                <Cell
                  key={step.label}
                  // Le refus est le seul état négatif de la série : sa teinte le signale, son
                  // libellé le nomme. Les trois autres étapes partagent l'accent, car elles
                  // mesurent une même progression.
                  fill={step.label.toLowerCase().startsWith("refus") ? CHART_INK.danger : CHART_INK.accent}
                />
              ))}
              <LabelList
                dataKey="count"
                position="right"
                className="fill-ink text-note font-semibold"
                offset={9}
              />
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>
      <ol className="sr-only">
        {steps.map((step) => (
          <li key={step.label}>
            {step.label} : {step.count} candidature{step.count > 1 ? "s" : ""}, {step.percentage} %
          </li>
        ))}
      </ol>
    </>
  );
}
