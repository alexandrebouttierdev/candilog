import type { MenuEntry } from "@/shared/ui";
import type { Application } from "@/shared/types/generated/applications";

/** Gestes possibles sur une candidature, câblés par l'écran. */
export interface ApplicationHandlers {
  readonly open: (application: Application) => void;
  readonly edit: (application: Application) => void;
  readonly openOffer: (application: Application) => void;
  readonly changeStatus: (application: Application) => void;
  readonly scheduleFollowUp: (application: Application) => void;
  readonly generateResume: (application: Application) => void;
  readonly generateLetter: (application: Application) => void;
  readonly analyzeResume: (application: Application) => void;
  readonly duplicate: (application: Application) => void;
  readonly remove: (application: Application) => void;
}

/**
 * Menu d'une candidature — liste fermée et ordonnée du design (`DECISIONS.md` B9), plus
 * « Modifier la fiche » (D4). Le même menu sert au clic droit, au bouton `⋯` de la ligne et
 * de l'inspecteur : un geste trouvé à un endroit se retrouve partout.
 */
export function applicationMenu(application: Application, handlers: ApplicationHandlers): MenuEntry[] {
  const item = (
    id: string,
    label: string,
    run: (application: Application) => void,
    extra: { shortcut?: string; tone?: "danger"; disabled?: boolean; reason?: string } = {},
  ): MenuEntry => ({ kind: "item", id, label, onSelect: () => run(application), ...extra });

  return [
    item("open", "Ouvrir", handlers.open, { shortcut: "enter" }),
    item("edit", "Modifier la fiche", handlers.edit, { shortcut: "mod+enter" }),
    item("offer", "Ouvrir l'offre", handlers.openOffer, {
      disabled: !application.job_url,
      reason: "aucun lien enregistré",
    }),
    { kind: "separator", id: "s1" },
    item("status", "Changer le statut…", handlers.changeStatus, { shortcut: "s" }),
    item("follow-up", "Programmer une relance…", handlers.scheduleFollowUp, { shortcut: "r" }),
    { kind: "separator", id: "s2" },
    item("resume", "Générer un CV ciblé", handlers.generateResume),
    item("letter", "Générer une lettre", handlers.generateLetter),
    item("analyze", "Analyser le CV face à l'offre", handlers.analyzeResume),
    { kind: "separator", id: "s3" },
    item("duplicate", "Dupliquer", handlers.duplicate, { shortcut: "mod+d" }),
    item("delete", "Supprimer…", handlers.remove, { shortcut: "mod+backspace", tone: "danger" }),
  ];
}
