import { useCallback, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import type { AiTask, Settings, TaskRoute } from "@/shared/types/generated/settings";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { Menu } from "@/shared/ui";
import type { MenuAnchor, MenuEntry } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { OTHER_PROVIDERS, idProvider, toProvider } from "../../model/providers";
import { AI_TASKS, assignmentOf, mainLabel } from "../../model/taskRouting";

/** Modèles proposés au sélecteur : installés localement, puis fournisseurs distants prêts. */
function choices(settings: Settings, models: readonly ManagedModelStatus[]) {
  const local: Array<{ route: TaskRoute; label: string; meta: string }> = models
    .filter((model) => model.installed)
    .map((model) => ({
      route: { provider: "candilog_local", model: model.definition.ollama_tag },
      label: model.definition.display_name,
      meta: `${(model.definition.approximate_download_bytes / 1e9).toFixed(1).replace(".", ",")} Go`,
    }));
  const ollama = idProvider(settings.llm.provider) === "ollama" ? settings.llm : settings.llm_presets.ollama;
  if (ollama?.model) {
    local.push({ route: { provider: "ollama", model: ollama.model }, label: `Ollama · ${ollama.model}`, meta: "" });
  }
  const remote = OTHER_PROVIDERS.filter((provider) => provider.id !== "ollama").flatMap((provider) => {
    const preset = idProvider(settings.llm.provider) === provider.id ? settings.llm : settings.llm_presets[provider.id];
    if (!preset?.model || !preset.api_key_configured) return [];
    return [{ route: { provider: toProvider(provider.id), model: preset.model }, label: `${provider.label} · ${preset.model}`, meta: "" }];
  });
  return { local, remote };
}

function sameRoute(a: TaskRoute | null | undefined, b: TaskRoute): boolean {
  return a !== null && a !== undefined && idProvider(a.provider) === idProvider(b.provider) && a.model === b.model;
}

const DOT = {
  local: "bg-st-g",
  remote: "bg-st-a",
  off: "bg-tx-7",
} as const;

/**
 * « Qui fait quoi » (`screens/13-ai-who-does-what.png`, `AI_TASK_ROUTING.md`) : chaque tâche
 * et le modèle qui la fait, avec sa localité (vert : rien ne sort ; ambre : envoi distant).
 *
 * Le choix est enregistré aussitôt. Une tâche sans modèle attitré suit le fournisseur
 * principal ; « Aucun » la désactive. Composite clavier : `↑ ↓` parcourent les tâches,
 * `⏎` ou `Espace` ouvrent le sélecteur.
 */
export function AiTaskRouting({
  settings,
  models,
  busy,
  onChange,
}: {
  settings: Settings;
  models: readonly ManagedModelStatus[];
  busy: boolean;
  onChange: (task: AiTask, route: TaskRoute | null | undefined) => void;
}) {
  const [open, setOpen] = useState<{ task: AiTask; anchor: MenuAnchor } | null>(null);
  const [focus, setFocus] = useState(0);
  const rows = useRef<Array<HTMLButtonElement | null>>([]);
  const { local, remote } = choices(settings, models);

  // Ligne d'où le sélecteur a été ouvert : le focus y revient à sa fermeture.
  const opener = useRef(0);
  const close = useCallback(() => {
    setOpen(null);
    rows.current[opener.current]?.focus();
  }, []);

  const openFor = (task: AiTask, element: HTMLElement) => {
    opener.current = AI_TASKS.findIndex((item) => item.value === task);
    const rect = element.getBoundingClientRect();
    setOpen({ task, anchor: new DOMRect(rect.right - 236, rect.bottom, 236, 0) });
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const next = Math.min(Math.max(focus + (event.key === "ArrowDown" ? 1 : -1), 0), AI_TASKS.length - 1);
    setFocus(next);
    rows.current[next]?.focus();
  };

  const task = open?.task ?? null;
  const current = task && task in settings.ai_routes ? settings.ai_routes[task] : undefined;
  const taskLabel = AI_TASKS.find((item) => item.value === task)?.label ?? "";
  const pick = (route: TaskRoute | null | undefined) => {
    if (task) onChange(task, route);
  };

  const entries: MenuEntry[] = [
    {
      kind: "item",
      id: "principal",
      label: `Fournisseur principal — ${mainLabel(settings, models)}`,
      checked: current === undefined,
      onSelect: () => pick(undefined),
    },
    { kind: "section", id: "local", label: "Sur votre ordinateur" },
    ...(local.length === 0
      ? [{ kind: "item" as const, id: "aucun-local", label: "Aucun modèle local installé", disabled: true, onSelect: () => undefined }]
      : local.map(
          (choice): MenuEntry => ({
            kind: "item",
            id: `local-${idProvider(choice.route.provider)}-${choice.route.model}`,
            label: choice.label,
            ...(choice.meta ? { meta: choice.meta } : {}),
            leading: <span aria-hidden className="size-1.5 flex-none rounded-full bg-st-g" />,
            checked: sameRoute(current, choice.route),
            onSelect: () => pick(choice.route),
          }),
        )),
    { kind: "section", id: "distant", label: "Services distants" },
    ...(remote.length === 0
      ? [{ kind: "item" as const, id: "aucun-distant", label: "Aucun fournisseur distant configuré", disabled: true, onSelect: () => undefined }]
      : remote.map(
          (choice): MenuEntry => ({
            kind: "item",
            id: `distant-${idProvider(choice.route.provider)}-${choice.route.model}`,
            label: choice.label,
            leading: <span aria-hidden className="size-1.5 flex-none rounded-full bg-st-a" />,
            checked: sameRoute(current, choice.route),
            onSelect: () => pick(choice.route),
          }),
        )),
    { kind: "separator", id: "sep" },
    { kind: "item", id: "aucun", label: "Aucun — désactiver cette tâche", checked: current === null, onSelect: () => pick(null) },
  ];

  return (
    <section aria-label="Qui fait quoi" className="mt-5">
      <div className="mb-2 flex items-baseline gap-3">
        <h2 className="caps">Qui fait quoi</h2>
        <span className="ml-auto text-tiny text-tx-6">Chaque tâche peut utiliser un modèle différent</span>
      </div>
      <div role="list" onKeyDown={onKeyDown}>
        {AI_TASKS.map((item, index) => {
          const assignment = assignmentOf(item.value, settings, models);
          return (
            <div role="listitem" key={item.value}>
              <button
                ref={(element) => {
                  rows.current[index] = element;
                }}
                type="button"
                disabled={busy}
                tabIndex={index === focus ? 0 : -1}
                aria-label={`${item.label}, ${assignment.locality === "off" ? "désactivée" : `modèle ${assignment.label}, ${assignment.locality === "local" ? "sur votre ordinateur" : "distant"}`} — modifier`}
                onFocus={() => setFocus(index)}
                onClick={(event) => openFor(item.value, event.currentTarget)}
                className={cn(
                  "-mx-2.5 flex h-[33px] w-[calc(100%+20px)] items-center gap-3 rounded-r7 px-2.5 text-left hover:bg-hover",
                  "focus-visible:bg-hover focus-visible:shadow-[inset_2px_0_0_var(--tx-6)] focus-visible:outline-none",
                  open?.task === item.value && "bg-hover",
                )}
              >
                <span className="min-w-0 flex-1 truncate text-ui text-tx-2">{item.label}</span>
                <span className="flex min-w-0 max-w-[55%] items-center gap-[7px] rounded-r6 bg-chip px-2 py-[3px]">
                  <span aria-hidden className={cn("size-1.5 flex-none rounded-full", DOT[assignment.locality])} />
                  <span className={cn("truncate text-small", assignment.locality === "off" ? "text-tx-6" : "text-tx-3")}>
                    {assignment.label}
                  </span>
                  {assignment.isDefault ? <span className="flex-none text-tiny text-tx-6">par défaut</span> : null}
                  <span aria-hidden className="flex-none text-tiny text-tx-6">
                    ›
                  </span>
                </span>
              </button>
            </div>
          );
        })}
      </div>
      <Menu
        open={open !== null}
        anchor={open?.anchor ?? null}
        label={`Modèle pour « ${taskLabel} »`}
        width={280}
        entries={entries}
        onClose={close}
        header={<div className="caps px-[9px] pt-1 pb-1.5">POUR « {taskLabel.toUpperCase()} »</div>}
      />
    </section>
  );
}
