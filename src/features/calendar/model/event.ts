import type { Interview } from "@/features/interviews/services/interviewService";
import type { FollowUp } from "@/features/followups/services/followUpService";
import { interviewIcon } from "@/features/interviews/model/types";
import { followUpIcon } from "@/features/followups/model/types";
import { timeFromTimestamp, dayOf } from "@/shared/lib/dates";
import type { Tone } from "@/shared/ui";
import type { IconName } from "@/shared/ui/icon-names";

/**
 * Événement du calendrier, entretien ou relance ramenés à une forme commune.
 *
 * La grille n'a pas à connaître deux entités : elle affiche des pastilles datées. Le type
 * d'origine reste accessible pour rouvrir la bonne modale au clic.
 */
export interface CalendarEvent {
  readonly id: string;
  readonly kind: "interview" | "follow_up";
  /** Day `AAAA-MM-JJ` de regroupement. */
  readonly day: string;
  /** Time `HH:MM`, absente pour une relance qui se programme au jour. */
  readonly time: string | null;
  readonly label: string;
  readonly detail: string | null;
  readonly icon: IconName;
  readonly tone: Tone;
}

/** Convertit un entretien en événement. Tonalité verte : c'est un avancement. */
export function fromInterview(interview: Interview): CalendarEvent {
  return {
    id: interview.id,
    kind: "interview",
    day: dayOf(interview.interview_date),
    time: timeFromTimestamp(interview.interview_date),
    label: interview.application_job_title ?? "Entretien",
    detail: interview.company_name,
    icon: interviewIcon(interview.type),
    tone: "success",
  };
}

/** Convertit une relance en événement. Tonalité ambre : c'est une action à mener. */
export function fromFollowUp(follow_up: FollowUp): CalendarEvent {
  return {
    id: follow_up.id,
    kind: "follow_up",
    day: dayOf(follow_up.follow_up_date),
    time: null,
    label: follow_up.application_job_title ?? "Relance",
    detail: follow_up.company_name,
    icon: followUpIcon(follow_up.type),
    tone: "warning",
  };
}

/**
 * Regroupe les événements par jour, chaque journée triée par heure.
 *
 * Les relances, sans heure, passent en tête de journée : elles se traitent quand on veut,
 * là où un entretien a un créneau.
 */
export function groupByDay(
  events: readonly CalendarEvent[],
): Map<string, CalendarEvent[]> {
  const parDay = new Map<string, CalendarEvent[]>();
  for (const event of events) {
    const journee = parDay.get(event.day);
    if (journee) {
      journee.push(event);
    } else {
      parDay.set(event.day, [event]);
    }
  }
  for (const journee of parDay.values()) {
    journee.sort((a, b) => (a.time ?? "").localeCompare(b.time ?? ""));
  }
  return parDay;
}
