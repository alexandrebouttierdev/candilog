import type { HistoryEntry, RelationKind } from "@/shared/types/generated/relations";

/** Ligne d'historique de l'inspecteur : date courte, fait, candidature concernée. */
export interface HistoryLine {
  readonly key: string;
  /** `JJ-MM`, dans le fuseau de la machine. */
  readonly day: string;
  /** Date complète, pour l'infobulle. */
  readonly fullDate: string;
  readonly text: string;
  /** Intitulé de la candidature concernée, sous le fait. */
  readonly job: string | null;
  /** Présent pour une note : elle seule se supprime. */
  readonly noteId: string | null;
}

const pad = (value: number) => String(value).padStart(2, "0");

/** Date du jour `AAAA-MM-JJ`, dans le fuseau de la machine. */
export function localToday(now = new Date()): string {
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/**
 * Jour local d'une entrée. Une date seule (`AAAA-MM-JJ`) est déjà locale ; un horodatage
 * RFC 3339 passe par `Date`, faute de quoi un statut changé à 23 h 30 serait daté du
 * lendemain (UTC).
 */
function localDay(at: string): { year: number; month: number; day: number } {
  if (at.length === 10) {
    return { year: Number(at.slice(0, 4)), month: Number(at.slice(5, 7)), day: Number(at.slice(8, 10)) };
  }
  const date = new Date(at);
  return { year: date.getFullYear(), month: date.getMonth() + 1, day: date.getDate() };
}

const STATUS_TEXT: Record<string, string> = {
  EN_ATTENTE: "Remise en attente",
  RELANCEE: "Marquée relancée",
  ENTRETIEN: "Passée en entretien",
  REFUS: "Candidature refusée",
};

const INTERVIEW_FORMAT: Record<string, string> = {
  Présentiel: "sur place",
  Visio: "visioconférence",
  Téléphonique: "par téléphone",
  Technique: "technique",
  RH: "RH",
};

const FOLLOW_UP_CHANNEL: Record<string, string> = {
  Email: "par e-mail",
  Téléphone: "par téléphone",
  LinkedIn: "sur LinkedIn",
};

function describe(entry: HistoryEntry, relation: RelationKind, now: Date): string {
  const detail = entry.detail ?? "";
  switch (entry.kind) {
    case "note":
      return detail;
    case "application_sent":
      return "Candidature envoyée";
    case "status_changed":
      return STATUS_TEXT[detail] ?? `Statut : ${detail}`;
    case "interview": {
      const date = new Date(entry.at);
      const time = `${pad(date.getHours())}:${pad(date.getMinutes())}`;
      const format = INTERVIEW_FORMAT[detail];
      return `${date > now ? "Entretien prévu" : "Entretien"} à ${time}${format ? ` · ${format}` : ""}`;
    }
    case "follow_up_done": {
      const channel = FOLLOW_UP_CHANNEL[detail];
      return channel ? `Relance faite ${channel}` : "Relance faite";
    }
    case "added":
      return relation === "company" ? "Entreprise ajoutée au suivi" : "Contact ajouté";
  }
}

/** Met en mots l'historique renvoyé par le backend, dans l'ordre reçu (le plus récent d'abord). */
export function historyLines(
  entries: readonly HistoryEntry[],
  relation: RelationKind,
  now = new Date(),
): HistoryLine[] {
  return entries.map((entry, index) => {
    const { year, month, day } = localDay(entry.at);
    return {
      key: entry.note_id ?? `${entry.kind}-${entry.at}-${entry.application_id ?? ""}-${index}`,
      day: `${pad(day)}-${pad(month)}`,
      fullDate: `${pad(day)}-${pad(month)}-${year}`,
      text: describe(entry, relation, now),
      job: entry.kind === "note" || entry.kind === "added" ? null : entry.job_title,
      noteId: entry.note_id,
    };
  });
}
