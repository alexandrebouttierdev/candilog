import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Sections } from "@/app/router/routes";
import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";

interface CommandItem {
  id: string;
  label: string;
  hint?: string;
  icon: string;
  group: string;
  action: () => void;
}

/**
 * Palette de commandes (Cmd/Ctrl+K) : navigation et actions rapides.
 */
export function CommandPalette({ onClose }: { onClose: () => void }) {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const items = useMemo((): CommandItem[] => {
    const nav: CommandItem[] = Sections.flatMap((section) =>
      section.routes.map((route) => ({
        id: route.path,
        label: route.label,
        hint: section.long_label,
        icon: route.icon,
        group: "Navigation",
        action: () => {
          void navigate(route.path);
          onClose();
        },
      })),
    );

    const actions: CommandItem[] = [
      {
        id: "new-application",
        label: "Nouvelle candidature",
        icon: "add",
        group: "Actions",
        action: () => {
          void navigate("/tracking/applications?nouvelle=1");
          onClose();
        },
      },
      {
        id: "generate-resume",
        label: "Générer un CV",
        icon: "auto_awesome",
        group: "Actions",
        action: () => {
          void navigate("/documents/generate-resume");
          onClose();
        },
      },
      {
        id: "settings-ai",
        label: "Paramètres IA",
        icon: "smart_toy",
        group: "Actions",
        action: () => {
          void navigate("/settings/ai");
          onClose();
        },
      },
    ];

    return [...nav, ...actions];
  }, [navigate, onClose]);

  const filtered = useMemo(() => {
    const term = query.trim().toLowerCase();
    if (!term) return items;
    return items.filter(
      (item) =>
        item.label.toLowerCase().includes(term) ||
        (item.hint?.toLowerCase().includes(term) ?? false),
    );
  }, [items, query]);

  const groups = useMemo(() => {
    const map = new Map<string, CommandItem[]>();
    for (const item of filtered) {
      const list = map.get(item.group) ?? [];
      list.push(item);
      map.set(item.group, list);
    }
    return map;
  }, [filtered]);

  const flatFiltered = useMemo(() => filtered, [filtered]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const execute = useCallback(
    (item: CommandItem) => {
      item.action();
    },
    [],
  );

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setSelected((s) => Math.min(s + 1, flatFiltered.length - 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (event.key === "Enter" && flatFiltered[selected]) {
      event.preventDefault();
      execute(flatFiltered[selected]);
    } else if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  };

  let flatIndex = 0;

  return (
    <div
      className="fixed inset-0 z-[200] flex items-start justify-center pt-[12vh]"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="scrim-palette absolute inset-0" aria-hidden />
      <div
        className="palette-in glass-palette relative z-10 flex w-[620px] max-h-[60vh] flex-col overflow-hidden rounded-overlay border border-overlay-strong shadow-overlay"
        role="dialog"
        aria-label="Palette de commandes"
        onKeyDown={onKeyDown}
      >
        <div className="flex items-center gap-[11px] border-b border-glass-topbar px-4 py-3.5">
          <Icon name="bolt" size={20} className="text-accent" />
          <input
            ref={inputRef}
            type="search"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelected(0);
            }}
            placeholder="Rechercher une page, une action…"
            aria-label="Rechercher"
            className="min-w-0 flex-1 bg-transparent text-[15px] text-ink outline-none placeholder:text-ink-disabled"
          />
          <kbd className="kbd">Échap</kbd>
        </div>

        <div ref={listRef} className="min-h-0 flex-1 overflow-y-auto p-2">
          {flatFiltered.length === 0 ? (
            <p className="px-4 py-6 text-center text-body text-ink-faint">
              Aucune commande pour « {query} »
            </p>
          ) : (
            Array.from(groups.entries()).map(([group, groupItems]) => (
              <div key={group} className="mb-1">
                <p className="px-2 pb-1 pt-1.5 text-eyebrow uppercase tracking-[0.07em] text-ink-label">
                  {group}
                </p>
                {groupItems.map((item) => {
                  const index = flatIndex;
                  flatIndex += 1;
                  const isSelected = index === selected;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      onClick={() => execute(item)}
                      onMouseEnter={() => setSelected(index)}
                      className={cn(
                        "flex h-[34px] w-full items-center gap-[11px] rounded-tile px-[9px] text-left transition-colors duration-hover",
                        isSelected ? "bg-surface-hover" : "hover:bg-surface-hover",
                      )}
                    >
                      <Icon name={item.icon} size={17} className="text-ink-muted-2" />
                      <span className="min-w-0 flex-1 truncate text-body text-ink-strong">{item.label}</span>
                      {item.hint ? (
                        <span className="max-w-[200px] truncate text-label text-ink-disabled">{item.hint}</span>
                      ) : null}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        <div className="flex items-center justify-between border-t border-glass-topbar bg-surface-elevated/50 px-3.5 py-2 text-meta text-ink-disabled">
          <span>↑↓ naviguer · Entrée ouvrir</span>
          <span>Candilog</span>
        </div>
      </div>
    </div>
  );
}

/** Écoute Cmd/Ctrl+K pour ouvrir la palette. */
export function useCommandPalette() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key === "k") {
        event.preventDefault();
        setOpen((v) => !v);
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  return { open, setOpen, close: () => setOpen(false) };
}

/** Bouton de la topbar pour ouvrir la palette. */
export function CommandPaletteTrigger({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label="Rechercher ou exécuter"
      className="flex h-[29px] w-[220px] max-w-full flex-none items-center gap-2 rounded-control border border-line bg-fill px-2.5 text-note text-ink-faint transition-colors duration-hover hover:bg-fill-hover hover:text-ink-muted"
    >
      <Icon name="search" size={16} />
      <span className="min-w-0 flex-1 truncate text-left">Rechercher ou exécuter…</span>
      <span className="kbd">⌘K</span>
    </button>
  );
}
