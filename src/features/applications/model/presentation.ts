import type { Application, ApplicationChannel } from "@/shared/types/generated/applications";
import { daysFrom } from "@/shared/lib/dates";

/** Référence lisible d'une candidature : `CAN-007`, `CAN-142`. */
export function formatReference(referenceNumber: number): string {
  return `CAN-${String(referenceNumber).padStart(3, "0")}`;
}

/** Les quatre canaux du formulaire, dans l'ordre du design (« Trouvée via »). */
export const Channels: ReadonlyArray<{ value: ApplicationChannel; label: string; long: string }> = [
  { value: "OFFER", label: "Offre", long: "Offre publiée" },
  { value: "COMPANY_SITE", label: "Site", long: "Site de l'entreprise" },
  { value: "NETWORK", label: "Réseau", long: "Réseau" },
  { value: "SPONTANEOUS", label: "Spontanée", long: "Candidature spontanée" },
];

export function channelLabel(channel: ApplicationChannel): string {
  return Channels.find((item) => item.value === channel)?.long ?? channel;
}

/** Au-delà de ce délai sans réponse, une candidature est signalée (bandeau du Kanban). */
export const SILENCE_DAYS = 14;

/** Échéance affichée en pastille sur une ligne ou une carte. */
export interface Due {
  readonly kind: "interview" | "follow_up" | "silence";
  readonly label: string;
  /** Nom accessible complet, la pastille étant abrégée. */
  readonly title: string;
  readonly tone: "success" | "accent" | "danger";
}

/** `AAAA-MM-JJ` → `JJ-MM`, la notation courte des listes denses. */
export function shortDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}`;
}

/**
 * Échéance la plus utile d'une candidature, dans l'ordre du design : l'entretien à venir
 * (« ◷ auj. 14:30 »), sinon la prochaine relance (« ↻ 17-09 »), sinon le silence d'une
 * candidature qui attend une réponse depuis plus de 14 jours (« 56 j »). Aucune pastille
 * sinon : une ligne sans échéance n'a rien à signaler.
 */
export function dueOf(application: Application, today = new Date()): Due | null {
  if (application.next_interview_at) {
    const day = application.next_interview_at.slice(0, 10);
    const time = application.next_interview_at.slice(11, 16);
    const isToday = day === localIso(today);
    const when = isToday ? "auj." : shortDate(day);
    return {
      kind: "interview",
      label: `◷ ${when}${time ? ` ${time}` : ""}`,
      title: `Entretien ${isToday ? "aujourd'hui" : `le ${shortDate(day)}`}${time ? ` à ${time}` : ""}`,
      tone: "success",
    };
  }
  if (application.next_follow_up_date) {
    return {
      kind: "follow_up",
      label: `↻ ${shortDate(application.next_follow_up_date)}`,
      title: `Relance prévue le ${shortDate(application.next_follow_up_date)}`,
      tone: "accent",
    };
  }
  const attend = application.status === "EN_ATTENTE" || application.status === "RELANCEE";
  const days = daysFrom(application.sent_date);
  if (attend && days > SILENCE_DAYS) {
    return {
      kind: "silence",
      label: `${days} j`,
      title: `Sans réponse depuis ${days} jours`,
      tone: "danger",
    };
  }
  return null;
}

/** Date locale `AAAA-MM-JJ` : « aujourd'hui » est celui de l'utilisateur, pas l'UTC. */
export function localIso(date: Date): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}
