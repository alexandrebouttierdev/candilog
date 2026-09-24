import type { ReactNode } from "react";
import { Icon } from "@/shared/ui";
import type { IconName } from "@/shared/ui/icon-names";

export function SettingsCard({
  icon,
  title,
  hint,
  action,
  className,
  children,
}: {
  icon: IconName;
  title: string;
  hint?: string;
  /** Action alignée à droite du titre (ex. Tester la connexion). */
  action?: ReactNode;
  className?: string;
  children: ReactNode;
}) {
  return (
    <section
      className={
        className ??
        "min-w-0 overflow-hidden rounded-card border border-line bg-surface"
      }
    >
      <div className="border-b border-line px-[18px] py-[14px]">
        <div className="flex items-center gap-2">
          <Icon name={icon} size={17} className="flex-none text-ink-faint" />
          <h2 className="text-item font-semibold text-ink">{title}</h2>
          {action ? <div className="ml-auto flex flex-none items-center gap-2">{action}</div> : null}
        </div>
        {hint ? <p className="mt-1 ml-[25px] text-label text-ink-faint">{hint}</p> : null}
      </div>
      <div className="px-[18px] py-4">{children}</div>
    </section>
  );
}
