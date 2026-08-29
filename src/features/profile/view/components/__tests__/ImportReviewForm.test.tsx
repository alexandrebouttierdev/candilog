import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { describe, expect, it, vi } from "vitest";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import {
  countSelected,
  importProfileRequestSchema,
  previewToFormValues,
  type ImportProfileFormInput,
  type ImportProfileFormValues,
} from "../../../model/import-review.schema";
import { ImportReviewForm } from "../ImportReviewForm";

const preview = (): ImportProfilePreview => ({
  identity: [
    {
      id: "title",
      label: "Titre professionnel",
      proposed: "Lead",
      existing: "Dev",
      has_conflict: true,
    },
  ],
  experiences: [
    {
      id: "exp-0",
      proposed: {
        title: "Développeur Frontend Senior",
        company: "Lumen Interactive",
        location: null,
        start_date: "2022-03",
        end_date: null,
        current: true,
        description: "Lead",
      },
      existing: {
        title: "Développeur Frontend",
        company: "Lumen Interactive",
        location: null,
        start_date: "2022-03",
        end_date: null,
        current: true,
        description: null,
      },
      existing_index: 0,
      has_conflict: true,
    },
  ],
  skills: [
    {
      id: "skill-0",
      proposed: { name: "React" },
      existing: null,
      existing_index: null,
      has_conflict: false,
    },
  ],
  education: [],
  languages: [],
  projects: [],
  certifications: [],
  counts: {
    identity: 1,
    experiences: 1,
    skills: 1,
    education: 0,
    languages: 0,
    projects: 0,
    certifications: 0,
  },
});

function Host({
  onSelected,
  onSubmit = vi.fn(),
}: {
  onSelected?: (count: number) => void;
  onSubmit?: (values: ImportProfileFormValues) => void;
}) {
  const form = useForm<ImportProfileFormInput, unknown, ImportProfileFormValues>({
    resolver: zodResolver(importProfileRequestSchema),
    defaultValues: previewToFormValues(preview()),
  });
  useWatch({ control: form.control });
  const selected = countSelected(form.getValues());
  onSelected?.(selected);
  return (
    <>
      <ImportReviewForm
        preview={preview()}
        entries={[]}
        formId="review"
        form={form}
        onSubmit={onSubmit}
      />
      <button type="submit" form="review" disabled={selected === 0}>
        Importer les éléments sélectionnés
      </button>
    </>
  );
}

describe("ImportReviewForm", () => {
  it("affiche tout ce qui est coché, avec les conflits", () => {
    render(<Host />);
    expect(screen.getByText("Aperçu de l'import")).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Titre professionnel" })).toHaveValue("Lead");
    expect(screen.getByLabelText("Poste")).toHaveValue("Développeur Frontend Senior");
    expect(screen.getByRole("textbox", { name: "Compétence React" })).toHaveValue("React");
    expect(screen.getAllByText(/similaire existe déjà/).length).toBe(2);
    expect(screen.getAllByRole("radio", { name: /Conserver l'existant/ }).length).toBe(2);
    expect(screen.queryByRole("button", { name: /Corriger/ })).not.toBeInTheDocument();
  });

  it("permet de modifier une donnée sans décocher la ligne", async () => {
    render(<Host />);
    const poste = screen.getByLabelText("Poste");
    await userEvent.clear(poste);
    await userEvent.type(poste, "Staff engineer");
    expect(screen.getByDisplayValue("Staff engineer")).toBeInTheDocument();
    expect(
      screen.getByRole("checkbox", { name: "Importer Développeur Frontend Senior" }),
    ).toBeChecked();
  });

  it("retire de l'aperçu une section ignorée", async () => {
    render(<Host />);
    await userEvent.click(screen.getByRole("checkbox", { name: "Importer les expériences" }));
    expect(
      screen.getByRole("checkbox", { name: "Importer Développeur Frontend Senior" }),
    ).not.toBeChecked();
    expect(screen.queryByLabelText("Poste")).not.toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Titre professionnel" })).toBeInTheDocument();
  });

  it("désactive l'import si tout est ignoré", async () => {
    render(<Host />);
    await userEvent.click(screen.getByRole("checkbox", { name: "Importer toutes les données" }));
    expect(screen.getByText("Rien à importer")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Importer les éléments sélectionnés" })).toBeDisabled();
  });
});
