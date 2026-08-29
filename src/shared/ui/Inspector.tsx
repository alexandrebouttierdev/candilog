import { useCallback, useRef, useState, type ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";
import { IconButton } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";

const DEFAULT_WIDTH = 380;
const MIN_WIDTH = 320;
const MAX_WIDTH = 460;

/**
 * Panneau latéral inspector : glass, redimensionnable, dense.
 */
export function Inspector({
  open,
  title,
  subtitle,
  actions,
  headerExtra,
  onClose,
  children,
  defaultWidth = DEFAULT_WIDTH,
  className,
}: {
  open: boolean;
  title: string;
  subtitle?: string | undefined;
  actions?: ReactNode;
  headerExtra?: ReactNode;
  onClose: () => void;
  children: ReactNode;
  defaultWidth?: number;
  className?: string;
}) {
  const [width, setWidth] = useState(defaultWidth);
  const [dragging, setDragging] = useState(false);
  const startRef = useRef({ x: 0, w: defaultWidth });

  useDismissable({ open, onDismiss: onClose });

  const onResizeStart = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      startRef.current = { x: event.clientX, w: width };

      const onMove = (moveEvent: Event) => {
        const clientX = (moveEvent as globalThis.PointerEvent).clientX;
        const delta = startRef.current.x - clientX;
        const next = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startRef.current.w + delta));
        setWidth(next);
      };

      const onUp = () => {
        setDragging(false);
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
      };

      setDragging(true);
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
    },
    [width],
  );

  if (!open) return null;

  return (
    <aside
      aria-label={title}
      className={cn("glass-inspector relative flex flex-none flex-col border-l border-glass-inspector", className)}
      style={{ width }}
    >
      <div
        role="separator"
        aria-orientation="vertical"
        aria-label="Redimensionner l'inspecteur"
        onPointerDown={onResizeStart}
        onDoubleClick={() => setWidth(defaultWidth)}
        className={cn(
          "absolute inset-y-0 -left-[3px] z-10 w-[7px] cursor-col-resize touch-none",
          dragging && "bg-accent-focus/30",
        )}
      />

      <header className="flex-none border-b border-line-soft px-4 pt-3.5 pb-3">
        <div className="flex items-start gap-2.5">
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-[14.5px] font-semibold tracking-[-0.01em] text-ink">{title}</h2>
            {subtitle ? (
              <p className="mt-0.5 truncate text-note text-ink-subtle">{subtitle}</p>
            ) : null}
          </div>
          <IconButton icon="close" label="Fermer l'inspecteur" size={17} onClick={onClose} />
        </div>
        {headerExtra ? <div className="mt-2.5">{headerExtra}</div> : null}
        {actions ? <div className="mt-2.5 flex items-center gap-1.5">{actions}</div> : null}
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3.5">{children}</div>
    </aside>
  );
}

/** État vide de l'inspecteur lorsqu'aucun élément est sélectionné. */
export function InspectorEmpty({
  title = "Aucune sélection",
  description = "Sélectionnez un élément pour afficher son détail ici, sans quitter la liste.",
}: {
  title?: string;
  description?: string;
}) {
  return (
    <div className="flex h-full flex-col items-center justify-center px-6 py-10 text-center">
      <span className="mb-3 flex size-[46px] items-center justify-center rounded-overlay bg-surface-elevated">
        <Icon name="left_panel_open" size={23} className="text-ink-label" />
      </span>
      <p className="text-item font-semibold text-ink">{title}</p>
      <p className="mt-1.5 max-w-[230px] text-note text-ink-faint">{description}</p>
    </div>
  );
}

/** Label de section en capitales dans l'inspecteur. */
export function InspectorSectionLabel({ children }: { children: ReactNode }) {
  return (
    <p className="mb-[7px] text-eyebrow uppercase tracking-[0.07em] text-ink-label">{children}</p>
  );
}

/** Rangée libellé / valeur dense de l'inspecteur. */
export function InspectorRow({
  label,
  children,
  tone,
}: {
  label: string;
  children: ReactNode;
  tone?: "accent" | "muted" | undefined;
}) {
  return (
    <div className="flex items-start justify-between gap-3 border-t border-field py-[7px] first:border-t-0">
      <span className="flex-none text-note text-ink-subtle">{label}</span>
      <span
        className={cn(
          "min-w-0 flex-1 truncate text-right text-body",
          tone === "accent" ? "text-accent-text" : tone === "muted" ? "text-ink-faint" : "text-ink-strong",
        )}
      >
        {children}
      </span>
    </div>
  );
}
