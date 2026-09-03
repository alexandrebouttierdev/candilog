import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { ActivityWeek } from "@/shared/types/generated/analytics";
import { EmptyState } from "@/shared/ui";
import { AXIS_TICK, CHART_INK, PLOT_MARGIN } from "./chartTheme";
import { ChartTooltip } from "./ChartTooltip";
import { formatDate } from "../analyticsDates";

/**
 * Candidatures envoyées, semaine par semaine.
 *
 * Une seule série : pas de légende, la carte qui l'accueille la nomme déjà. Les valeurs
 * restent lisibles sans survol grâce à l'axe des ordonnées et à la liste équivalente
 * réservée aux lecteurs d'écran — l'infobulle n'est jamais le seul accès au chiffre.
 */
export function ActivityChart({
  activity,
  height = 150,
  shortLabels = false,
}: {
  activity: readonly ActivityWeek[];
  height?: number;
  /** Étiquettes numériques `JJ/MM`, pour les séries longues ou les cartes étroites. */
  shortLabels?: boolean;
}) {
  if (activity.every((week) => week.count === 0)) {
    return (
      <EmptyState
        icon="bar_chart"
        title="Pas encore d’activité"
        description="Les candidatures envoyées apparaîtront ici semaine après semaine."
      />
    );
  }

  const data = activity.map((week) => ({ ...week }));

  return (
    <>
      <div style={{ height }} role="img" aria-label="Candidatures envoyées par semaine">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={data} margin={PLOT_MARGIN} barCategoryGap="22%" accessibilityLayer>
            <CartesianGrid stroke={CHART_INK.grid} vertical={false} />
            <XAxis
              dataKey="start"
              tickFormatter={(value: string) => formatDate(value, shortLabels ? "numeric" : "court")}
              tick={AXIS_TICK}
              tickLine={false}
              axisLine={{ stroke: CHART_INK.grid }}
              interval="preserveStartEnd"
              minTickGap={8}
            />
            <YAxis
              width={26}
              allowDecimals={false}
              tick={AXIS_TICK}
              tickLine={false}
              axisLine={false}
            />
            <Tooltip
              cursor={{ fill: CHART_INK.empty }}
              content={({ active, payload }) => {
                const weekData = active
                  ? (payload?.[0]?.payload as ActivityWeek | undefined)
                  : undefined;
                if (!weekData) return null;
                return (
                  <ChartTooltip title={`Semaine du ${formatDate(weekData.start, "long")}`}>
                    {`${weekData.count} candidature${weekData.count > 1 ? "s" : ""}`}
                  </ChartTooltip>
                );
              }}
            />
            {/* `minPointSize` : une semaine sans candidature garde un socle visible, sinon
                elle disparaîtrait de la série au lieu d'y valoir zéro. */}
            <Bar
              dataKey="count"
              radius={[4, 4, 0, 0]}
              minPointSize={2}
              isAnimationActive={false}
            >
              {data.map((week) => (
                <Cell
                  key={week.start}
                  fill={week.count === 0 ? CHART_INK.empty : CHART_INK.accent}
                />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>
      <ol className="sr-only">
        {activity.map((week) => (
          <li key={week.start}>
            Semaine du {formatDate(week.start, "long")} : {week.count} candidature
            {week.count > 1 ? "s" : ""}
          </li>
        ))}
      </ol>
    </>
  );
}
