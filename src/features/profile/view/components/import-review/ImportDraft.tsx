import type { ReactNode } from "react";
import type { UseFormReturn } from "react-hook-form";
import type { ImportProfilePreview, ImportResolution } from "@/shared/types/generated/profile";
import { cn } from "@/shared/lib/cn";
import { Card, CardHeader, CardMeta, FormField, Icon, TextArea, TextInput } from "@/shared/ui";
import { IMPORT_SECTIONS, type ImportProfileFormInput, type ImportProfileFormValues, type ImportSection } from "../../../model/import-review.schema";
import { blockId, SECTION_ICONS, SECTION_LABELS, type CatalogRow } from "./catalog";

export function ImportDraft({
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
