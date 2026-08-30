import { useEffect, useState } from "react";
import { useWatch, type UseFormReturn } from "react-hook-form";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import { cn } from "@/shared/lib/cn";
import {
  EmptyState,
  ErrorBanner,
  Icon,
  SplitPane,
} from "@/shared/ui";
import {
  countChecked,
  IMPORT_SECTIONS,
  setSectionSelected,
  summarizeImport,
  type ImportProfileFormInput,
  type ImportProfileFormValues,
  type ImportSection,
} from "../../model/import-review.schema";
import type { ImportJournalEntry } from "../../viewmodel/useProfileImportProgress";
import { ImportJournal } from "./ImportJournal";
import { blockId, catalogOf, SECTION_ARIA, SECTION_ICONS, SECTION_LABELS } from "./import-review/catalog";
import { ImportDraft } from "./import-review/ImportDraft";

/** Revue maître-détail : cocher à gauche, aperçu éditable de l'import à droite. */
export function ImportReviewForm({
  preview,
  entries,
  formId,
  form,
  formError,
  onSubmit,
}: {
  preview: ImportProfilePreview;
  entries: ImportJournalEntry[];
  formId: string;
  form: UseFormReturn<ImportProfileFormInput, unknown, ImportProfileFormValues>;
  formError?: string | null;
  onSubmit: (values: ImportProfileFormValues) => void;
}) {
  useWatch({ control: form.control });
  const values = form.getValues();
  const catalog = catalogOf(preview);
  const [focusKey, setFocusKey] = useState(() => catalog[0]?.id ?? "");
  const focus = catalog.find((row) => row.id === focusKey) ?? catalog[0] ?? null;
  const checkedRows = catalog.filter((row) => values[row.section][row.index]?.selected);
  const checkedItems = checkedRows.length;

  useEffect(() => {
    if (!focusKey) return;
    document.getElementById(blockId(focusKey))?.scrollIntoView?.({ block: "nearest" });
  }, [focusKey]);

  const selectSection = (section: ImportSection, selected: boolean) => {
    const options = { shouldDirty: true, shouldTouch: true } as const;
    switch (section) {
      case "identity":
        form.setValue(
          "identity",
          form.getValues("identity").map((item) => ({ ...item, selected })),
          options,
        );
        break;
      case "experiences":
        form.setValue(
          "experiences",
          setSectionSelected(form.getValues("experiences"), selected),
          options,
        );
        break;
      case "skills":
        form.setValue("skills", setSectionSelected(form.getValues("skills"), selected), options);
        break;
      case "education":
        form.setValue(
          "education",
          setSectionSelected(form.getValues("education"), selected),
          options,
        );
        break;
      case "languages":
        form.setValue(
          "languages",
          setSectionSelected(form.getValues("languages"), selected),
          options,
        );
        break;
      case "projects":
        form.setValue("projects", setSectionSelected(form.getValues("projects"), selected), options);
        break;
      case "certifications":
        form.setValue(
          "certifications",
          setSectionSelected(form.getValues("certifications"), selected),
          options,
        );
        break;
    }
  };

  const selectAll = (selected: boolean) => {
    IMPORT_SECTIONS.forEach((section) => {
      if (form.getValues(section).length > 0) selectSection(section, selected);
    });
  };

  return (
    <form
      id={formId}
      onSubmit={(event) => {
        void form.handleSubmit(onSubmit)(event);
      }}
      className="flex h-full min-h-0 flex-1 flex-col overflow-hidden"
    >
      {formError ? (
        <div className="flex-none px-[18px] pt-3">
          <ErrorBanner title="Import impossible" message={formError} />
        </div>
      ) : null}
      <SplitPane
        defaultLeftWidth={260}
        minLeft={220}
        maxLeft={340}
        minRight={360}
        left={
          <div className="flex min-h-0 flex-1 flex-col bg-surface">
            <div className="flex flex-none items-center gap-2.5 border-b border-line px-[14px] py-[11px]">
              <TriStateCheckbox
                checked={checkedItems === catalog.length && catalog.length > 0}
                indeterminate={checkedItems > 0 && checkedItems < catalog.length}
                label="Importer toutes les données"
                onChange={selectAll}
              />
              <span className="min-w-0 flex-1 truncate text-body font-semibold text-ink">
                À importer
              </span>
              <span className="tabular text-label text-ink-faint">
                {checkedItems}/{catalog.length}
              </span>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto px-[9px] py-2">
              {IMPORT_SECTIONS.map((section) => {
                const rows = catalog.filter((row) => row.section === section);
                if (rows.length === 0) return null;
                const checked = countChecked(values[section]);
                return (
                  <section key={section} className="mb-5">
                    <div className="mb-2.5 flex items-center gap-2 px-1.5">
                      <TriStateCheckbox
                        checked={checked === rows.length}
                        indeterminate={checked > 0 && checked < rows.length}
                        label={`Importer ${SECTION_ARIA[section]}`}
                        onChange={(next) => selectSection(section, next)}
                      />
                      <Icon
                        name={SECTION_ICONS[section]}
                        size={14}
                        className="flex-none text-ink-faint"
                      />
                      <h3 className="min-w-0 flex-1 truncate text-eyebrow font-semibold tracking-wide text-ink-faint uppercase">
                        {SECTION_LABELS[section]}
                      </h3>
                      <span className="tabular text-meta text-ink-faint">
                        {checked}/{rows.length}
                      </span>
                    </div>
                    <div className="pl-1.5">
                    {rows.map((row) => {
                      const selected = values[row.section][row.index]?.selected ?? false;
                      return (
                        <div
                          key={row.id}
                          className={cn(
                            "mb-1 flex items-center gap-1.5 rounded-tile border pr-1.5",
                            focus?.id === row.id
                              ? "border-accent-border bg-accent-tint"
                              : "border-transparent hover:bg-neutral-tint",
                            selected ? "" : "opacity-55",
                          )}
                        >
                          <input
                            type="checkbox"
                            className="ml-2.5 flex-none"
                            aria-label={`Importer ${row.title}`}
                            {...form.register(
                              `${row.section}.${row.index}.selected`,
                              { onChange: () => setFocusKey(row.id) },
                            )}
                            onClick={(event) => event.stopPropagation()}
                          />
                          <button
                            type="button"
                            aria-label={row.title}
                            aria-current={focus?.id === row.id ? "true" : undefined}
                            onClick={() => setFocusKey(row.id)}
                            className="min-w-0 flex-1 py-[9px] pr-1.5 text-left"
                          >
                            <span className="flex items-start gap-1">
                              <span className="min-w-0 flex-1">
                                <span className="block truncate text-item font-semibold text-ink">
                                  {row.title}
                                </span>
                                {row.subtitle ? (
                                  <span className="mt-px block truncate text-label text-ink-faint">
                                    {row.subtitle}
                                  </span>
                                ) : null}
                              </span>
                              {row.conflict ? (
                                <Icon
                                  name="warning"
                                  size={14}
                                  className="mt-0.5 flex-none text-warning"
                                />
                              ) : null}
                            </span>
                          </button>
                        </div>
                      );
                    })}
                    </div>
                  </section>
                );
              })}
            </div>
          </div>
        }
        right={
          <div className="flex min-h-0 flex-1 flex-col bg-page">
            <div className="flex flex-none items-baseline justify-between gap-3 border-b border-line bg-surface px-[22px] py-[11px]">
              <p className="text-body font-semibold text-ink">Aperçu de l'import</p>
              <p className="tabular text-meta text-ink-faint">
                {checkedItems === 0
                  ? "Aucun élément"
                  : `${checkedItems} élément${checkedItems > 1 ? "s" : ""}`}
              </p>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto px-[18px] py-4">
              {checkedRows.length === 0 ? (
                <EmptyState
                  bordered
                  icon="playlist_add_check"
                  title="Rien à importer"
                  description="Cochez des éléments à gauche pour les voir ici et les corriger avant enregistrement."
                />
              ) : (
                <ImportDraft
                  preview={preview}
                  rows={checkedRows}
                  focusId={focus?.id ?? ""}
                  form={form}
                />
              )}
            </div>
            <div className="flex-none bg-surface px-[18px] pb-3">
              <ImportJournal entries={entries} />
            </div>
          </div>
        }
      />
      <p className="hidden" data-replaced-count={summarizeImport(values, preview).replaced} />
    </form>
  );
}

function TriStateCheckbox({
  checked,
  indeterminate,
  label,
  onChange,
}: {
  checked: boolean;
  indeterminate: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <input
      type="checkbox"
      checked={checked}
      aria-label={label}
      ref={(node) => {
        if (node) node.indeterminate = indeterminate;
      }}
      onClick={(event) => event.stopPropagation()}
      onChange={(event) => onChange(event.target.checked)}
    />
  );
}
