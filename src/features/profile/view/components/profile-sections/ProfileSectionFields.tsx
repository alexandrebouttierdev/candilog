import type { ReactNode } from "react";
import type { UseFormRegisterReturn } from "react-hook-form";
import { Button, FormField, Select, TextArea, TextInput } from "@/shared/ui";

export function ProfileField({
  label,
  registration,
  error,
  required = false,
  type = "text",
  placeholder,
  disabled = false,
}: {
  label: string;
  registration: UseFormRegisterReturn;
  error?: string | undefined;
  required?: boolean | undefined;
  type?: string | undefined;
  placeholder?: string | undefined;
  disabled?: boolean | undefined;
}) {
  return (
    <FormField label={label} required={required} error={error}>
      {(props) => (
        <TextInput
          {...props}
          {...registration}
          type={type}
          placeholder={placeholder}
          disabled={disabled}
          invalid={Boolean(error)}
        />
      )}
    </FormField>
  );
}

export function ProfileArea({
  label,
  registration,
  error,
  help,
  rows,
}: {
  label: string;
  registration: UseFormRegisterReturn;
  error?: string | undefined;
  help?: string | undefined;
  rows?: number | undefined;
}) {
  return (
    <FormField label={label} error={error} help={help}>
      {(props) => (
        <TextArea
          {...props}
          {...registration}
          rows={rows}
          invalid={Boolean(error)}
        />
      )}
    </FormField>
  );
}

export function ProfileSelect({
  label,
  registration,
  error,
  children,
}: {
  label: string;
  registration: UseFormRegisterReturn;
  error?: string | undefined;
  children: ReactNode;
}) {
  return (
    <FormField label={label} required error={error}>
      {(props) => (
        <Select {...props} {...registration} invalid={Boolean(error)}>
          {children}
        </Select>
      )}
    </FormField>
  );
}

export function RepeatList({
  empty,
  addLabel,
  onAdd,
  children,
}: {
  empty: string;
  addLabel: string;
  onAdd: () => void;
  children: ReactNode;
}) {
  return (
    <div className="space-y-4">
      {children || <EmptyInline text={empty} />}
      <Button variant="secondary" icon="add" onClick={onAdd}>
        {addLabel}
      </Button>
    </div>
  );
}

export function ItemCard({
  title,
  onRemove,
  children,
}: {
  title: string;
  onRemove: () => void;
  children: ReactNode;
}) {
  return (
    <fieldset className="rounded-card border border-line bg-surface-alt p-4">
      <legend className="sr-only">{title}</legend>
      <div className="mb-4 flex items-center gap-2">
        <p className="min-w-0 flex-1 truncate text-section text-ink">{title}</p>
        <Button
          variant="ghost"
          icon="delete"
          aria-label={`Supprimer ${title}`}
          onClick={onRemove}
        >
          Supprimer
        </Button>
      </div>
      {children}
    </fieldset>
  );
}

function EmptyInline({ text }: { text: string }) {
  return (
    <p className="rounded-card border border-dashed border-line bg-surface-alt px-4 py-8 text-center text-body text-ink-muted">
      {text}
    </p>
  );
}
