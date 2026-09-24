import type { ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { PATHS } from "@/shared/lib/paths";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { Kbd } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/**
 * Surcouche des générateurs (`screens/15-generator-resume.png`, `16-generator-letter.png`) :
 * elle occupe toute la fenêtre, se ferme par la croix ou `Échap` (`INTERACTIONS.md` §1), et
 * range le travail en trois colonnes — réglages à gauche, feuille A4 au centre sur fond
 * gris, déroulé et recommandations à droite.
 */
export function GeneratorFrame({
  title,
  actions,
  left,
  right,
  children,
  status,
  keys,
  closeDisabled = false,
  onSubmit,
  onSave,
}: {
  title: string;
  actions: ReactNode;
  left: ReactNode;
  right: ReactNode;
  children: ReactNode;
  /** Barre d'état, à gauche : ce que fait la surcouche maintenant. */
  status: string;
  /** Rappels clavier de la barre d'état, à droite. */
  keys?: ReadonlyArray<{ label: string; shortcut: string }>;
  /** Une génération en cours se termine ou s'arrête avant de fermer. */
  closeDisabled?: boolean;
  /** `⌘⏎` : l'action principale (générer). */
  onSubmit?: () => void;
  /** `⌘S` : enregistrer le document. */
  onSave?: () => void;
}) {
  const navigate = useNavigate();
  const close = () => {
    if (!closeDisabled) void navigate(PATHS.documents);
  };
  // Les raccourcis d'écran se taisent sous une surface : la surcouche porte les siens.
  useDismissable({
    open: true,
    onDismiss: close,
    dismissDisabled: closeDisabled,
    ...(onSubmit ? { onSubmit } : {}),
  });

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title}
      onKeyDown={(event) => {
        if (onSave && (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
          event.preventDefault();
          onSave();
        }
      }}
      className="fixed inset-0 z-50 flex flex-col bg-app"
    >
      <header data-tauri-drag-region className="flex h-titlebar flex-none items-center gap-3 border-b border-bd-soft pr-3 pl-[84px]">
        <button
          type="button"
          aria-label="Fermer"
          aria-keyshortcuts="Escape"
          disabled={closeDisabled}
          onClick={close}
          className="flex size-[22px] flex-none items-center justify-center rounded-r6 bg-chip text-tiny text-tx-4 hover:text-tx disabled:opacity-50"
        >
          ✕
        </button>
        <p className="flex min-w-0 items-center gap-2 text-ui">
          <span className="text-tx-4">Documents</span>
          <span aria-hidden className="text-tx-6">
            ›
          </span>
          <span className="truncate font-medium text-tx">{title}</span>
        </p>
        <div className="ml-auto flex flex-none items-center gap-1.5">{actions}</div>
      </header>

      <div className="flex min-h-0 flex-1">
        <aside className="w-[260px] flex-none overflow-y-auto border-r border-bd-soft bg-app px-3 pt-3 pb-5">{left}</aside>
        <main className="flex min-w-0 flex-1 flex-col overflow-auto bg-canvas">{children}</main>
        <aside className="hidden w-[270px] flex-none overflow-y-auto border-l border-bd-soft bg-app px-3.5 pt-3 pb-5 wide:block">
          {right}
        </aside>
      </div>

      <footer className="flex h-statusbar flex-none items-center gap-3 border-t border-bd-soft px-3.5">
        <span className="min-w-0 truncate font-mono text-caps text-tx-5">{status}</span>
        <span className="ml-auto flex flex-none items-center gap-3 text-sub text-tx-3">
          {(keys ?? []).map((key) => (
            <span key={key.label} className="flex items-center gap-1.5">
              {key.label}
              <Kbd shortcut={key.shortcut} tone="chip" decorative />
            </span>
          ))}
          <span className="flex items-center gap-1.5">
            Fermer
            <Kbd shortcut="escape" tone="chip" decorative />
          </span>
        </span>
      </footer>
    </div>
  );
}

/** Section d'un panneau latéral : en-tête capitalisé, contenu dessous. */
export function PaneSection({
  title,
  aside,
  children,
  className,
}: {
  title: string;
  aside?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section aria-label={title} className={cn("mb-4", className)}>
      <h2 className="caps mb-2 flex items-center">
        {title}
        {aside ? <span className="ml-auto font-mono normal-case tracking-normal text-tx-6">{aside}</span> : null}
      </h2>
      {children}
    </section>
  );
}

/** Étape du déroulé d'une génération, à droite de la surcouche. */
export interface GeneratorStep {
  readonly label: string;
  readonly state: "pending" | "running" | "done";
  /** Durée mesurée d'une étape terminée. */
  readonly ms: number | null;
}

export function StepList({ steps }: { steps: readonly GeneratorStep[] }) {
  return (
    <ol className="flex flex-col gap-2.5">
      {steps.map((step) => (
        <li key={step.label} className="flex items-center gap-2.5 text-small">
          <span
            aria-hidden
            className={cn(
              "size-[9px] flex-none rounded-full",
              step.state === "done" && "bg-st-g",
              step.state === "running" && "animate-pulse border-[1.5px] border-ac bg-tint-ac-bg",
              step.state === "pending" && "border-[1.5px] border-tx-6",
            )}
          />
          <span className={step.state === "pending" ? "text-tx-4" : "text-tx-2"}>{step.label}</span>
          {step.ms !== null ? (
            <span className="ml-auto font-mono text-caps text-tx-5">{(step.ms / 1000).toFixed(1).replace(".", ",")} s</span>
          ) : null}
          <span className="sr-only">
            {step.state === "done" ? "terminée" : step.state === "running" ? "en cours" : "à venir"}
          </span>
        </li>
      ))}
    </ol>
  );
}
