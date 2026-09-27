import { useCallback, useRef, useState, type ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { usePointerDrag } from "@/shared/hooks/usePointerDrag";

/**
 * Panneaux redimensionnables horizontaux (workspace desktop).
 *
 * Zone de saisie 7 px, curseur col-resize, double-clic pour réinitialiser la largeur.
 */
export function SplitPane({
  left,
  right,
  defaultLeftWidth = 280,
  minLeft = 200,
  maxLeft = 480,
  minRight = 240,
  className,
}: {
  left: ReactNode;
  right: ReactNode;
  defaultLeftWidth?: number;
  minLeft?: number;
  maxLeft?: number;
  minRight?: number;
  className?: string;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [leftWidth, setLeftWidth] = useState(defaultLeftWidth);
  const [dragging, setDragging] = useState(false);

  const dragStart = useRef({ x: 0, width: defaultLeftWidth });
  const startPointerDrag = usePointerDrag(
    (moveEvent) => {
      const container = containerRef.current;
      if (!container) return;
      const delta = moveEvent.clientX - dragStart.current.x;
      const maxAllowed = container.offsetWidth - minRight - 7;
      const next = Math.min(maxLeft, Math.max(minLeft, dragStart.current.width + delta));
      setLeftWidth(Math.min(next, maxAllowed));
    },
    () => setDragging(false),
  );
  const onPointerDown = useCallback(
    (event: React.PointerEvent) => {
      dragStart.current = { x: event.clientX, width: leftWidth };
      setDragging(true);
      startPointerDrag(event);
    },
    [leftWidth, startPointerDrag],
  );

  return (
    <div ref={containerRef} className={cn("flex min-h-0 min-w-0 flex-1", className)}>
      <div className="flex min-h-0 min-w-0 flex-col" style={{ width: leftWidth, flexShrink: 0 }}>
        {left}
      </div>
      <div
        role="separator"
        aria-orientation="vertical"
        aria-label="Redimensionner les panneaux"
        onPointerDown={onPointerDown}
        onDoubleClick={() => setLeftWidth(defaultLeftWidth)}
        className={cn(
          "group relative z-10 w-[7px] flex-none cursor-col-resize touch-none",
          dragging && "bg-accent-focus/40",
        )}
      >
        <div
          className={cn(
            "absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-line transition-colors",
            "group-hover:bg-line-strong",
            dragging && "bg-accent-focus",
          )}
        />
      </div>
      <div className="flex min-h-0 min-w-0 flex-1 flex-col">{right}</div>
    </div>
  );
}
