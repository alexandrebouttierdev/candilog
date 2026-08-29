import { NavLink, useLocation } from "react-router-dom";
import type { SectionDef } from "@/app/router/routes";
import { cn } from "@/shared/lib/cn";
import { Icon } from "@/shared/ui/Icon";

/** Sous-navigation 186 px, items 30 px. */
export function SubNav({ section }: { section: SectionDef }) {
  const { pathname } = useLocation();

  if (section.routes.length <= 1) return null;

  return (
    <nav
      aria-label={section.long_label}
      className="glass-subnav flex w-subnav flex-none flex-col border-r border-glass-subnav py-3"
    >
      <p className="mb-2 px-[18px] text-eyebrow uppercase text-ink-label">{section.short_label}</p>
      <ul className="flex flex-col gap-px px-2.5">
        {section.routes.map((route) => {
          const selected =
            route.path === "/" ? pathname === "/" : pathname.startsWith(route.path);
          return (
            <li key={route.path}>
              <NavLink
                to={route.path}
                className={cn(
                  "flex h-[30px] items-center gap-2 rounded-control px-2 text-body transition-colors duration-hover",
                  selected
                    ? "bg-accent-tint-12 font-semibold text-accent-text-soft"
                    : "text-ink-tertiary hover:bg-surface-hover",
                )}
              >
                <Icon name={route.icon} size={16} className={selected ? "text-accent-text" : "text-ink-subtle"} />
                <span className="min-w-0 truncate">{route.label}</span>
              </NavLink>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
