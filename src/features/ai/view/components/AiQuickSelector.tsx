import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { cn } from "@/shared/lib/cn";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { Icon, StatusPill } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import type { ManagedModelId } from "@/shared/types/generated/ai";
import type { LlmForm } from "@/shared/types/generated/settings";
import { AppError } from "@/shared/types/app-error";
import {
  ManagedPublisherLogo,
  PROVIDERS,
  aiStatus,
  defaultEndpoint,
  defaultModel,
  getProvider,
  idProvider,
  isAiConfigured,
  managedOllamaStatus,
  providerLogo,
  toProvider,
  type ProviderOption,
} from "@/features/settings";
import { useAiQuickSelectorViewModel } from "../../viewmodel/useAiQuickSelectorViewModel";

const DOT: Record<Tone, string> = {
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-danger",
  neutral: "bg-ink-subtle",
  accent: "bg-accent",
};

function providerStatusTone(configured: boolean): Tone {
  return configured ? "success" : "warning";
}

/** Sélecteur rapide global : fournisseur + modèle, synchronisé avec les réglages. */
export function AiQuickSelector({ shellBrand = false }: { shellBrand?: boolean }) {
  const navigate = useNavigate();
  const root = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [focusedProvider, setFocusedProvider] = useState<ProviderOption["id"] | null>(null);
  const vm = useAiQuickSelectorViewModel();
  const llm = vm.settings?.llm;
  const activeProviderId = llm ? idProvider(llm.provider) : null;
  const managed = { data: vm.managed };
  const activateManaged = (modelId: ManagedModelId) => {
    vm.activateManaged(modelId);
    setOpen(false);
  };

  useDismissable({ open, onDismiss: () => setOpen(false) });

  useEffect(() => {
    if (!open) return;
    const onPointer = (event: MouseEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onPointer);
    return () => document.removeEventListener("mousedown", onPointer);
  }, [open]);

  const columnProvider = focusedProvider ?? activeProviderId ?? "candilog_local";

  const activeModelLabel = useMemo(() => {
    if (!llm || !activeProviderId) return "";
    if (activeProviderId === "candilog_local") {
      return managed.data?.active_model?.display_name ?? "";
    }
    return llm.model.trim();
  }, [activeProviderId, llm, managed.data]);

  const fournisseur = llm ? getProvider(llm.provider) : PROVIDERS[0]!;

  const globalEtat = useMemo(() => {
    if (!llm || !activeProviderId) {
      return { label: "Non configuré", tone: "neutral" as Tone };
    }
    if (activeProviderId === "candilog_local") {
      return managedOllamaStatus(managed.data ?? null, null);
    }
    return aiStatus(llm, "idle");
  }, [activeProviderId, llm, managed.data]);

  const logo = activeProviderId ? providerLogo(activeProviderId) : null;

  const selectProvider = async (id: ProviderOption["id"]) => {
    if (!vm.settings) return;
    if (id === "candilog_local") {
      const active = managed.data?.active_model;
      if (active) {
        activateManaged(active.id);
        return;
      }
      setOpen(false);
      void navigate("/settings/ai");
      return;
    }
    const nextLlm: LlmForm = {
      ...vm.settings.llm,
      provider: toProvider(id),
      endpoint: defaultEndpoint(id),
      model: id === activeProviderId ? vm.settings.llm.model : defaultModel(id),
    };
    if (!isAiConfigured(nextLlm)) {
      setOpen(false);
      void navigate("/settings/ai");
      return;
    }
    await vm.saveSettings({ ...vm.settings, llm: nextLlm });
    setOpen(false);
  };

  const selectManagedModel = (modelId: ManagedModelId, installed: boolean) => {
    if (!installed) {
      setOpen(false);
      void navigate("/settings/ai");
      return;
    }
    activateManaged(modelId);
  };

  const triggerLabel = activeModelLabel
    ? `${fournisseur.label} · ${activeModelLabel}`
    : fournisseur.label;

  return (
    <div ref={root} className="relative">
      <button
        type="button"
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-label={`Fournisseur IA : ${triggerLabel} — ${globalEtat.label}`}
        onClick={() => setOpen((value) => !value)}
        className={cn(
          "inline-flex h-control max-w-[min(42vw,18rem)] items-center gap-1.5 rounded-button border px-2.5",
          "text-item font-medium transition-colors duration-hover",
          shellBrand
            ? "shell-brand-control focus-visible:outline-1 focus-visible:outline-rail-focus"
            : "border-control-strong bg-fill text-ink hover:bg-fill-hover focus-visible:outline-1 focus-visible:outline-accent-focus",
        )}
      >
        <span className="relative flex size-5 flex-none items-center justify-center">
          {logo ? (
            <img
              src={logo.src}
              alt=""
              width={16}
              height={16}
              className={cn("size-4 object-contain", logo.mono && "dark:invert")}
            />
          ) : (
            <Icon
              name="smart_toy"
              size={16}
              className={shellBrand ? "text-rail-ink" : "text-ink-muted"}
            />
          )}
          <span
            aria-hidden
            className={cn(
              "absolute -right-0.5 -bottom-0.5 size-1.5 rounded-full ring-2",
              shellBrand ? "ring-rail-item-active-border" : "ring-page",
              DOT[globalEtat.tone],
            )}
          />
        </span>
        <span className="min-w-0 truncate">{triggerLabel}</span>
        <Icon
          name="expand_more"
          size={15}
          className={cn("flex-none", shellBrand ? "text-rail-ink" : "text-ink-faint")}
        />
      </button>

      {open ? (
        <div
          role="dialog"
          aria-label="Choisir le fournisseur et le modèle IA"
          className="glass-popover absolute top-[calc(100%+6px)] right-0 z-50 flex max-h-[min(70vh,480px)] w-[min(92vw,560px)] overflow-hidden rounded-overlay border border-overlay shadow-overlay"
        >
          <div className="flex w-[220px] flex-none flex-col overflow-y-auto border-r border-line-soft p-2">
            <p className="px-2 py-1 text-eyebrow uppercase text-ink-label">Fournisseur</p>
            {PROVIDERS.map((item) => {
              const selected = item.id === columnProvider;
              const itemConfigured =
                item.id === "candilog_local"
                  ? Boolean(managed.data?.active_model)
                  : vm.settings
                    ? isAiConfigured({
                        ...vm.settings.llm,
                        provider: toProvider(item.id),
                      })
                    : false;
              const logoItem = providerLogo(item.id);
              return (
                <button
                  key={item.id}
                  type="button"
                  onMouseEnter={() => setFocusedProvider(item.id)}
                  onFocus={() => setFocusedProvider(item.id)}
                  onClick={() => void selectProvider(item.id)}
                  className={cn(
                    "flex w-full items-start gap-2 rounded-button px-2 py-2 text-left transition-colors",
                    selected ? "bg-accent-tint text-accent" : "hover:bg-fill-hover",
                  )}
                >
                  <span className="mt-0.5 flex size-6 flex-none items-center justify-center rounded-control bg-surface">
                    {logoItem ? (
                      <img
                        src={logoItem.src}
                        alt=""
                        width={14}
                        height={14}
                        className={cn("size-3.5", logoItem.mono && "dark:invert")}
                      />
                    ) : (
                      <Icon name="smart_toy" size={14} className="text-ink-muted" />
                    )}
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-label font-mid">{item.label}</span>
                    <span className="block truncate text-meta text-ink-faint">{item.hint}</span>
                    <StatusPill tone={providerStatusTone(itemConfigured)} className="mt-1">
                      {itemConfigured ? "Configuré" : "À configurer"}
                    </StatusPill>
                  </span>
                  {item.id === activeProviderId ? (
                    <Icon name="check" size={16} className="mt-1 flex-none text-accent" />
                  ) : null}
                </button>
              );
            })}
          </div>
          <div className="min-w-0 flex-1 overflow-y-auto p-2">
            <p className="px-2 py-1 text-eyebrow uppercase text-ink-label">Modèles</p>
            <ModelColumn
              providerId={columnProvider}
              llm={llm}
              managed={managed.data}
              onSelectManaged={selectManagedModel}
              onConfigure={() => {
                setOpen(false);
                void navigate("/settings/ai");
              }}
              onSelectRemote={async (model) => {
                if (!vm.settings || !llm) return;
                await vm.saveSettings({
                  ...vm.settings,
                  llm: { ...llm, model, provider: toProvider(columnProvider) },
                });
                setOpen(false);
              }}
            />
          </div>
        </div>
      ) : null}
    </div>
  );
}

function ModelColumn({
  providerId,
  llm,
  managed,
  onSelectManaged,
  onConfigure,
  onSelectRemote,
}: {
  providerId: ProviderOption["id"];
  llm: LlmForm | undefined;
  managed: ReturnType<typeof useAiQuickSelectorViewModel>["managed"];
  onSelectManaged: (id: ManagedModelId, installed: boolean) => void;
  onConfigure: () => void;
  onSelectRemote: (model: string) => Promise<void>;
}) {
  if (providerId === "candilog_local") {
    if (!managed) {
      return <p className="px-2 py-3 text-note text-ink-muted">Chargement…</p>;
    }
    return (
      <ul className="flex flex-col gap-1">
        {managed.models.map((item) => (
          <li key={item.definition.id}>
            <button
              type="button"
              disabled={!item.installed && item.machine_fit === "insufficient_memory"}
              onClick={() => onSelectManaged(item.definition.id, item.installed)}
              className={cn(
                "flex w-full items-center gap-2 rounded-button px-2 py-2 text-left",
                item.active ? "bg-accent-tint" : "hover:bg-fill-hover",
              )}
            >
              <span className="flex size-6 flex-none items-center justify-center rounded-control bg-surface">
                <ManagedPublisherLogo
                  publisher={item.definition.publisher}
                  className="size-3.5"
                />
              </span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-label font-mid text-ink">
                  {item.definition.display_name}
                </span>
                <span className="block truncate text-meta text-ink-faint">
                  {item.definition.publisher_label}
                  {item.installed ? " · Installé" : " · Télécharger"}
                </span>
              </span>
              {item.active ? (
                <Icon name="check" size={16} className="flex-none text-accent" />
              ) : null}
            </button>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <RemoteModelList
      providerId={providerId}
      llm={llm}
      onConfigure={onConfigure}
      onSelectRemote={onSelectRemote}
    />
  );
}

function RemoteModelList({
  providerId,
  llm,
  onConfigure,
  onSelectRemote,
}: {
  providerId: ProviderOption["id"];
  llm: LlmForm | undefined;
  onConfigure: () => void;
  onSelectRemote: (model: string) => Promise<void>;
}) {
  const vm = useAiQuickSelectorViewModel();
  const query = useQuery({
    queryKey: ["parametres", "modeles-quick", providerId, llm?.endpoint, llm?.model],
    queryFn: () => vm.listModels(providerId, llm!),
    enabled: Boolean(llm),
  });

  if (query.isPending) return <p className="px-2 py-3 text-note text-ink-muted">Chargement des modèles…</p>;
  if (query.error) {
    const message =
      query.error instanceof AppError ? query.error.message : "Modèles inaccessibles.";
    return (
      <div className="px-2 py-3">
        <p className="text-note text-danger">{message}</p>
        <button type="button" onClick={onConfigure} className="mt-2 text-label font-semibold text-accent">
          Configurer
        </button>
      </div>
    );
  }
  const models = query.data ?? [];
  if (models.length === 0) {
    return (
      <div className="px-2 py-3">
        <p className="text-note text-ink-muted">Aucun modèle disponible.</p>
        <button type="button" onClick={onConfigure} className="mt-2 text-label font-semibold text-accent">
          Configurer
        </button>
      </div>
    );
  }

  return (
    <ul className="flex flex-col gap-1">
      {models.map((model) => (
        <li key={model}>
          <button
            type="button"
            onClick={() => void onSelectRemote(model)}
            className={cn(
              "flex w-full items-center justify-between gap-2 rounded-button px-2 py-2 text-left hover:bg-fill-hover",
              llm?.model === model && idProvider(llm.provider) === providerId ? "bg-accent-tint" : "",
            )}
          >
            <span className="truncate font-mono text-label text-ink">{model}</span>
            {llm?.model === model && idProvider(llm.provider) === providerId ? (
              <Icon name="check" size={16} className="text-accent" />
            ) : null}
          </button>
        </li>
      ))}
    </ul>
  );
}
