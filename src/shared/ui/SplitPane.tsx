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
  const dragStart = useRef<{ side: "left" | "right"; x: number; left: number; right: number }>({
    side: "left",
    x: 0,
    left: 0,
    right: 0,
  });

  const startPointerDrag = usePointerDrag(
    (moveEvent) => {
      const container = containerRef.current;
      if (!container) return;
      const { side, x, left, right } = dragStart.current;
      const delta = moveEvent.clientX - x;

      if (side === "left") {
        const maxAllowed = container.offsetWidth - rightWidth - minCenter - 14;
        const next = Math.min(maxLeft, Math.max(minLeft, left + delta));
        setLeftWidth(Math.min(next, maxAllowed));
      } else {
        const maxAllowed = container.offsetWidth - leftWidth - minCenter - 14;
        const next = Math.min(maxRight, Math.max(minRight, right - delta));
        setRightWidth(Math.min(next, maxAllowed));
      }
    },
    () => setDragging(null),
  );

  const startDrag = useCallback(
    (side: "left" | "right", event: React.PointerEvent) => {
      dragStart.current = { side, x: event.clientX, left: leftWidth, right: rightWidth };
      setDragging(side);
      startPointerDrag(event);
    },
    [leftWidth, rightWidth, startPointerDrag],
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
