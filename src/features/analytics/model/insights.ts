import type { Analytics } from "@/shared/types/generated/analytics";
import { channelLabel } from "@/features/applications";

/** Constat affiché sous « Ce que disent ces chiffres ». */
export interface Insight {
  readonly tone: "g" | "a";
  readonly text: string;
}

function plural(count: number, one: string, many: string): string {
  return `${count} ${count > 1 ? many : one}`;
}

/**
 * Constats tirés des chiffres de la période (`screens/10-analytics.png`). Chacun se vérifie
 * dans les blocs voisins : aucune projection, aucune promesse — un constat qui ne repose sur
 * rien de mesuré n'est pas écrit.
 */
export function insightsOf(data: Analytics): Insight[] {
  const insights: Insight[] = [];
  const { metrics } = data;
  if (metrics.applications === 0) return insights;

  // Le canal le mieux répondu, dès qu'au moins deux canaux ont assez d'envois pour comparer.
  const comparable = data.channels.filter((rate) => rate.applications >= 2);
  if (comparable.length >= 2) {
    const rate = (item: (typeof comparable)[number]) => item.responses / item.applications;
    const best = [...comparable].sort((a, b) => rate(b) - rate(a))[0]!;
    const worst = [...comparable].sort((a, b) => rate(a) - rate(b))[0]!;
    if (rate(best) > rate(worst)) {
      insights.push({
        tone: "g",
        text: `« ${channelLabel(best.channel)} » répond le mieux : ${Math.round(rate(best) * 100)} % de réponses (${best.responses}/${best.applications}), contre ${Math.round(rate(worst) * 100)} % pour « ${channelLabel(worst.channel)} ».`,
      });
    }
  }

  if (data.to_follow_up.length > 0) {
    insights.push({
      tone: "a",
      text: `${plural(data.to_follow_up.length, "candidature attend", "candidatures attendent")} une réponse depuis plus d'une semaine : relancez-les, ou clôturez celles qui ne reviendront pas.`,
    });
  }

  if (metrics.responses > 0) {
    insights.push({
      tone: "g",
      text: `${plural(metrics.interviews, "réponse", "réponses")} sur ${metrics.responses} ${metrics.interviews > 1 ? "ont" : "a"} mené à un entretien.`,
    });
  } else {
    insights.push({
      tone: "a",
      text: `Aucune réponse sur ${plural(metrics.applications, "candidature envoyée", "candidatures envoyées")} pour l'instant.`,
    });
  }
  return insights;
}
