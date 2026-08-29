import { useCallback, useRef, useState, type ReactNode } from "react";
import { cn } from "@/shared/lib/cn";

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

  const onPointerDown = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      const container = containerRef.current;
      if (!container) return;

      const startX = event.clientX;
      const startWidth = leftWidth;

      const onMove = (moveEvent: Event) => {
        const clientX = (moveEvent as globalThis.PointerEvent).clientX;
        const containerWidth = container.offsetWidth;
        const delta = clientX - startX;
        const maxAllowed = containerWidth - minRight - 7;
        const next = Math.min(maxLeft, Math.max(minLeft, startWidth + delta));
        setLeftWidth(Math.min(next, maxAllowed));
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
    [leftWidth, minLeft, maxLeft, minRight],
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

/**
 * Trois panneaux redimensionnables : gauche | centre | droite.
 */
export function TripleSplitPane({
  left,
  center,
  right,
  defaultLeftWidth = 280,
  defaultRightWidth = 320,
  minLeft = 200,
  maxLeft = 400,
  minCenter = 320,
  minRight = 240,
  maxRight = 460,
  className,
}: {
  left: ReactNode;
  center: ReactNode;
  right: ReactNode;
  defaultLeftWidth?: number;
  defaultRightWidth?: number;
  minLeft?: number;
  maxLeft?: number;
  minCenter?: number;
  minRight?: number;
  maxRight?: number;
  className?: string;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [leftWidth, setLeftWidth] = useState(defaultLeftWidth);
  const [rightWidth, setRightWidth] = useState(defaultRightWidth);
  const [dragging, setDragging] = useState<"left" | "right" | null>(null);

  const startDrag = useCallback(
    (side: "left" | "right", event: React.PointerEvent) => {
      event.preventDefault();
      const container = containerRef.current;
      if (!container) return;

      const startX = event.clientX;
      const startLeft = leftWidth;
      const startRight = rightWidth;

      const onMove = (moveEvent: Event) => {
        const clientX = (moveEvent as globalThis.PointerEvent).clientX;
        const containerWidth = container.offsetWidth;
        const delta = clientX - startX;

        if (side === "left") {
          const maxAllowed = containerWidth - rightWidth - minCenter - 14;
          const next = Math.min(maxLeft, Math.max(minLeft, startLeft + delta));
          setLeftWidth(Math.min(next, maxAllowed));
        } else {
          const maxAllowed = containerWidth - leftWidth - minCenter - 14;
          const next = Math.min(maxRight, Math.max(minRight, startRight - delta));
          setRightWidth(Math.min(next, maxAllowed));
        }
      };

      const onUp = () => {
        setDragging(null);
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
      };

      setDragging(side);
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
    },
    [leftWidth, rightWidth, minLeft, maxLeft, minCenter, minRight, maxRight],
  );

  return (
    <div ref={containerRef} className={cn("flex min-h-0 min-w-0 flex-1", className)}>
      <div className="flex min-h-0 min-w-0 flex-col" style={{ width: leftWidth, flexShrink: 0 }}>
        {left}
      </div>
      <Divider
        dragging={dragging === "left"}
        onPointerDown={(e) => startDrag("left", e)}
        onDoubleClick={() => setLeftWidth(defaultLeftWidth)}
      />
      <div className="flex min-h-0 min-w-0 flex-1 flex-col">{center}</div>
      <Divider
        dragging={dragging === "right"}
        onPointerDown={(e) => startDrag("right", e)}
        onDoubleClick={() => setRightWidth(defaultRightWidth)}
      />
      <div className="flex min-h-0 min-w-0 flex-col" style={{ width: rightWidth, flexShrink: 0 }}>
        {right}
      </div>
    </div>
  );
}

function Divider({
  dragging,
  onPointerDown,
  onDoubleClick,
}: {
  dragging: boolean;
  onPointerDown: (e: React.PointerEvent) => void;
  onDoubleClick: () => void;
}) {
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label="Redimensionner les panneaux"
      onPointerDown={onPointerDown}
      onDoubleClick={onDoubleClick}
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
  );
}
