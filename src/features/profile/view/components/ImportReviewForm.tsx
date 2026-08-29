import { useEffect, useState, type ReactNode } from "react";
import { useWatch, type UseFormReturn } from "react-hook-form";
import type { ImportProfilePreview, ImportResolution } from "@/shared/types/generated/profile";
import { cn } from "@/shared/lib/cn";
import {
  Card,
  CardHeader,
  CardMeta,
  EmptyState,
  ErrorBanner,
  FormField,
  Icon,
  SplitPane,
  TextArea,
  TextInput,
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

const SECTION_LABELS: Record<ImportSection, string> = {
  identity: "Informations personnelles",
  experiences: "Expériences",
  skills: "Compétences",
  education: "Formations",
  languages: "Langues",
  projects: "Projets",
  certifications: "Certifications",
};

const SECTION_ARIA: Record<ImportSection, string> = {
  identity: "les informations personnelles",
  experiences: "les expériences",
  skills: "les compétences",
  education: "les formations",
  languages: "les langues",
  projects: "les projets",
  certifications: "les certifications",
};

const SECTION_ICONS: Record<ImportSection, string> = {
  identity: "badge",
  experiences: "work_history",
  skills: "psychology",
  education: "school",
  languages: "translate",
  projects: "rocket_launch",
  certifications: "workspace_premium",
};

type CatalogRow = {
  section: ImportSection;
  index: number;
  id: string;
  title: string;
  subtitle?: string;
  conflict: boolean;
};

function catalogOf(preview: ImportProfilePreview): CatalogRow[] {
  return [
    ...preview.identity.map((item, index) => ({
      section: "identity" as const,
      index,
      id: item.id,
      title: item.label,
      conflict: item.has_conflict,
    })),
    ...preview.experiences.map((item, index) => ({
      section: "experiences" as const,
      index,
      id: item.id,
      title: item.proposed.title,
      subtitle: `${item.proposed.company}${item.proposed.start_date ? ` · ${item.proposed.start_date}` : ""}`,
      conflict: item.has_conflict,
    })),
    ...preview.skills.map((item, index) => ({
      section: "skills" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      conflict: item.has_conflict,
    })),
    ...preview.education.map((item, index) => ({
      section: "education" as const,
      index,
      id: item.id,
      title: item.proposed.degree,
      subtitle: item.proposed.school,
      conflict: item.has_conflict,
    })),
    ...preview.languages.map((item, index) => ({
      section: "languages" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      conflict: item.has_conflict,
    })),
    ...preview.projects.map((item, index) => ({
      section: "projects" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      ...(item.proposed.technologies ? { subtitle: item.proposed.technologies } : {}),
      conflict: item.has_conflict,
    })),
    ...preview.certifications.map((item, index) => ({
      section: "certifications" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      ...(item.proposed.issuer ? { subtitle: item.proposed.issuer } : {}),
      conflict: item.has_conflict,
    })),
  ];
}

function blockId(id: string) {
  return `import-block-${id}`;
}

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
                              `${row.section}.${row.index}.selected` as `${ImportSection}.${number}.selected`,
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

function ImportDraft({
  preview,
  rows,
  focusId,
  form,
}: {
  preview: ImportProfilePreview;
  rows: CatalogRow[];
  focusId: string;
  form: UseFormReturn<ImportProfileFormInput, unknown, ImportProfileFormValues>;
}) {
  return (
    <div className="space-y-4">
      {IMPORT_SECTIONS.map((section) => {
        const sectionRows = rows.filter((row) => row.section === section);
        if (sectionRows.length === 0) return null;
        return (
          <Card key={section} clipped>
            <CardHeader compact icon={SECTION_ICONS[section]} meta={<CardMeta>{sectionRows.length}</CardMeta>}>
              {SECTION_LABELS[section]}
            </CardHeader>
            <div
              className={
                section === "skills"
                  ? "flex flex-wrap gap-2 px-[17px] py-3"
                  : "divide-y divide-line"
              }
            >
              {sectionRows.map((row) => (
                <PreviewBlock
                  key={row.id}
                  preview={preview}
                  row={row}
                  focused={focusId === row.id}
                  form={form}
                />
              ))}
            </div>
          </Card>
        );
      })}
    </div>
  );
}

function PreviewBlock({
  preview,
  row,
  focused,
  form,
}: {
  preview: ImportProfilePreview;
  row: CatalogRow;
  focused: boolean;
  form: UseFormReturn<ImportProfileFormInput, unknown, ImportProfileFormValues>;
}) {
  const errors = form.formState.errors;
  const { existing, proposed, allowAdd } = comparisonOf(preview, row);
  const compact = row.section === "skills" && !row.conflict;

  return (
    <article
      id={blockId(row.id)}
      className={cn(
        compact ? "min-w-[140px] flex-1" : "px-[17px] py-3.5",
        focused && !compact ? "bg-accent-tint-08" : "",
      )}
    >
      {row.conflict ? (
        <p className="mb-2 flex items-start gap-1 text-meta text-warning">
          <Icon name="warning" size={14} className="mt-0.5 flex-none" />
          Une entrée similaire existe déjà.
        </p>
      ) : null}
      {row.conflict && existing ? <Comparison existing={existing} proposed={proposed} /> : null}
      {row.conflict ? (
        <Resolution
          name={`${row.section}.${row.index}.resolution`}
          register={form.register}
          allowAdd={allowAdd}
        />
      ) : null}
      <div className={row.conflict ? "mt-3 space-y-3" : compact ? "" : "space-y-3"}>
        {fieldsOf(preview, row, form, errors, compact)}
      </div>
    </article>
  );
}

function fieldsOf(
  preview: ImportProfilePreview,
  row: CatalogRow,
  form: UseFormReturn<ImportProfileFormInput, unknown, ImportProfileFormValues>,
  errors: UseFormReturn<ImportProfileFormInput>["formState"]["errors"],
  compact: boolean,
): ReactNode {
  const index = row.index;
  switch (row.section) {
    case "identity": {
      const item = preview.identity[index];
      if (!item) return null;
      return (
        <FormField label={item.label} error={errors.identity?.[index]?.value?.message}>
          {(props) =>
            item.id === "resume" ? (
              <TextArea {...props} rows={4} {...form.register(`identity.${index}.value`)} />
            ) : (
              <TextInput {...props} {...form.register(`identity.${index}.value`)} />
            )
          }
        </FormField>
      );
    }
    case "experiences":
      return (
        <>
          <div className="grid grid-cols-2 gap-2">
            <FormField label="Poste" error={errors.experiences?.[index]?.value?.title?.message}>
              {(props) => (
                <TextInput {...props} {...form.register(`experiences.${index}.value.title`)} />
              )}
            </FormField>
            <FormField
              label="Entreprise"
              error={errors.experiences?.[index]?.value?.company?.message}
            >
              {(props) => (
                <TextInput {...props} {...form.register(`experiences.${index}.value.company`)} />
              )}
            </FormField>
            <FormField
              label="Date de début"
              error={errors.experiences?.[index]?.value?.start_date?.message}
            >
              {(props) => (
                <TextInput {...props} {...form.register(`experiences.${index}.value.start_date`)} />
              )}
            </FormField>
            <FormField label="Date de fin">
              {(props) => (
                <TextInput {...props} {...form.register(`experiences.${index}.value.end_date`)} />
              )}
            </FormField>
          </div>
          <FormField label="Description">
            {(props) => (
              <TextArea
                {...props}
                rows={3}
                {...form.register(`experiences.${index}.value.description`)}
              />
            )}
          </FormField>
        </>
      );
    case "skills":
      return compact ? (
        <TextInput
          aria-label={`Compétence ${row.title}`}
          {...form.register(`skills.${index}.value.name`)}
        />
      ) : (
        <FormField label="Compétence" error={errors.skills?.[index]?.value?.name?.message}>
          {(props) => <TextInput {...props} {...form.register(`skills.${index}.value.name`)} />}
        </FormField>
      );
    case "education":
      return (
        <div className="grid grid-cols-2 gap-2">
          <FormField label="Diplôme" error={errors.education?.[index]?.value?.degree?.message}>
            {(props) => (
              <TextInput {...props} {...form.register(`education.${index}.value.degree`)} />
            )}
          </FormField>
          <FormField
            label="Établissement"
            error={errors.education?.[index]?.value?.school?.message}
          >
            {(props) => (
              <TextInput {...props} {...form.register(`education.${index}.value.school`)} />
            )}
          </FormField>
        </div>
      );
    case "languages":
      return (
        <div className="grid grid-cols-2 gap-2">
          <FormField label="Langue" error={errors.languages?.[index]?.value?.name?.message}>
            {(props) => (
              <TextInput {...props} {...form.register(`languages.${index}.value.name`)} />
            )}
          </FormField>
          <FormField label="Niveau" error={errors.languages?.[index]?.value?.level?.message}>
            {(props) => (
              <TextInput {...props} {...form.register(`languages.${index}.value.level`)} />
            )}
          </FormField>
        </div>
      );
    case "projects":
      return (
        <div className="space-y-3">
          <FormField label="Nom" error={errors.projects?.[index]?.value?.name?.message}>
            {(props) => <TextInput {...props} {...form.register(`projects.${index}.value.name`)} />}
          </FormField>
          <FormField label="Lien">
            {(props) => <TextInput {...props} {...form.register(`projects.${index}.value.url`)} />}
          </FormField>
        </div>
      );
    case "certifications":
      return (
        <div className="space-y-3">
          <FormField label="Nom" error={errors.certifications?.[index]?.value?.name?.message}>
            {(props) => (
              <TextInput {...props} {...form.register(`certifications.${index}.value.name`)} />
            )}
          </FormField>
          <FormField label="Lien">
            {(props) => (
              <TextInput {...props} {...form.register(`certifications.${index}.value.url`)} />
            )}
          </FormField>
        </div>
      );
  }
}

function comparisonOf(
  preview: ImportProfilePreview,
  row: CatalogRow,
): { existing: string[] | null; proposed: string[]; allowAdd: boolean } {
  switch (row.section) {
    case "identity": {
      const item = preview.identity[row.index];
      return {
        existing: item?.existing ? [item.existing] : null,
        proposed: item ? [item.proposed] : [],
        allowAdd: false,
      };
    }
    case "experiences": {
      const item = preview.experiences[row.index];
      return {
        existing: item?.existing
          ? [item.existing.title, item.existing.company, item.existing.start_date]
          : null,
        proposed: item
          ? [item.proposed.title, item.proposed.company, item.proposed.start_date]
          : [],
        allowAdd: true,
      };
    }
    case "skills": {
      const item = preview.skills[row.index];
      return {
        existing: item?.existing ? [item.existing.name] : null,
        proposed: item ? [item.proposed.name] : [],
        allowAdd: true,
      };
    }
    case "education": {
      const item = preview.education[row.index];
      return {
        existing: item?.existing ? [item.existing.degree, item.existing.school] : null,
        proposed: item ? [item.proposed.degree, item.proposed.school] : [],
        allowAdd: true,
      };
    }
    case "languages": {
      const item = preview.languages[row.index];
      return {
        existing: item?.existing ? [item.existing.name] : null,
        proposed: item ? [item.proposed.name] : [],
        allowAdd: true,
      };
    }
    case "projects": {
      const item = preview.projects[row.index];
      return {
        existing: item?.existing ? [item.existing.name] : null,
        proposed: item ? [item.proposed.name] : [],
        allowAdd: true,
      };
    }
    case "certifications": {
      const item = preview.certifications[row.index];
      return {
        existing: item?.existing ? [item.existing.name] : null,
        proposed: item ? [item.proposed.name] : [],
        allowAdd: true,
      };
    }
  }
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

function Comparison({ existing, proposed }: { existing: string[]; proposed: string[] }) {
  return (
    <div className="mb-2 grid grid-cols-[1fr_auto_1fr] items-start gap-2 text-meta">
      <div>
        <p className="text-eyebrow text-ink-faint uppercase">Existant</p>
        {existing.map((line, index) => (
          <p key={line} className={line !== proposed[index] ? "text-accent" : "text-ink-muted"}>
            {line}
          </p>
        ))}
      </div>
      <span className="text-ink-faint">→</span>
      <div>
        <p className="text-eyebrow text-ink-faint uppercase">CV</p>
        {proposed.map((line, index) => (
          <p key={line} className={line !== existing[index] ? "text-accent" : "text-ink-muted"}>
            {line}
          </p>
        ))}
      </div>
    </div>
  );
}

function Resolution({
  name,
  register,
  allowAdd,
}: {
  name: `${ImportSection}.${number}.resolution`;
  register: UseFormReturn<ImportProfileFormInput>["register"];
  allowAdd: boolean;
}) {
  const options: Array<{ value: ImportResolution; label: string }> = [
    { value: "keep_existing", label: "Conserver l'existant" },
    { value: "replace", label: "Remplacer par le CV" },
  ];
  if (allowAdd) options.push({ value: "add_as_new", label: "Ajouter en plus" });
  return (
    <fieldset className="flex flex-wrap gap-x-4 gap-y-1">
      {options.map((option) => (
        <label key={option.value} className="flex items-center gap-1.5 text-meta text-ink-muted">
          <input type="radio" value={option.value} {...register(name)} />
          {option.label}
        </label>
      ))}
    </fieldset>
  );
}
