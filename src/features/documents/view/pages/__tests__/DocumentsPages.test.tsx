import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { documentsService } from "../../../services/documentsService";
import { workspaceFixture } from "../../../model/resumeWorkspace";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { useUiStore } from "@/shared/lib/ui-store";
import { ResumeLibraryPage } from "../ResumeLibraryPage";
import { ResumeGeneratorPage } from "../ResumeGeneratorPage";
import { ResumeAnalysisPage } from "../ResumeAnalysisPage";
import { LetterWriterPage } from "../LettersPages";
import { AppError } from "@/shared/types/app-error";
import { aiService } from "@/features/ai/services/aiService";
import { useAiOperationStore } from "@/features/ai/viewmodel/ai-operation-store";

const navigateMock = vi.hoisted(() => vi.fn());
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual<typeof import("react-router-dom")>("react-router-dom");
  return { ...actual, useNavigate: () => navigateMock };
});

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}><MemoryRouter>{children}</MemoryRouter></QueryClientProvider>;
}

function aiExecution<T>(output: T) {
  return { output, elapsed_ms: 18_400, tokens_used: 1_024 };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useAiOperationStore.setState({ active: null });
  navigateMock.mockReset();
  useUiStore.setState({ toasts: [] });
});

describe("bibliothèque de CV paginée", () => {
  it("recherche et change de page côté backend sans charger la liste exhaustive", async () => {
    const exhaustive = vi.spyOn(documentsService, "listResume").mockResolvedValue([]);
    const paged = vi.spyOn(documentsService, "listResumePage").mockImplementation(({ page, page_size, search }) => Promise.resolve({
      items: [{ id: `${search || "cv"}-${page}`, name: `${search || "CV"} page ${page}`, created_at: "2026-08-30T00:00:00Z" }],
      total: search ? 1 : 9,
      page,
      page_size,
      total_pages: search ? 1 : 2,
    }));
    vi.spyOn(documentsService, "getResume").mockImplementation((id) => Promise.resolve({ id, name: id, content: null, created_at: "2026-08-30T00:00:00Z" }));

    render(<ResumeLibraryPage />, { wrapper });
    await waitFor(() => expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 8, search: "" }));

    await userEvent.click(screen.getByRole("button", { name: "Page suivante" }));
    await waitFor(() => expect(paged).toHaveBeenCalledWith({ page: 2, page_size: 8, search: "" }));

    const search = screen.getByPlaceholderText("Rechercher une version…");
    await userEvent.type(search, "cible");
    await waitFor(() => expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 8, search: "cible" }));
    expect(exhaustive).not.toHaveBeenCalled();
  });
});

describe("analyse explicite d'un CV sélectionné", () => {
  it("sélectionne le PDF sans analyser puis transmet son chemin au clic dédié", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({
      path: "/tmp/cv.pdf",
      name: "cv.pdf",
    });
    const analyze = vi.spyOn(aiService, "analyzeResume").mockResolvedValue({
      output: {
        resume: { resume: "Profil", experiences: [], skills: [], education: [] },
        job_offer: {
          title: "Développeur",
          skills: [],
          soft_skills: [],
          experience: null,
          keywords: [],
        },
        score: {
          total: 72,
          skills: null,
          experience: null,
          ats: null,
          present: [],
          missing: [],
        },
        analysis: { recap: "Analyse terminée", recommendations: [], content_recommendations: [] },
      },
      elapsed_ms: 18_400,
      tokens_used: 1_024,
    });

    render(<ResumeAnalysisPage />, { wrapper });
    expect(screen.getByText("Comparez un CV à l’offre ciblée")).toBeInTheDocument();
    expect(screen.queryByText("Lecture locale")).not.toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/Offre ciblée/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: "Choisir un fichier" }));

    expect(analyze).not.toHaveBeenCalled();
    expect(await screen.findByText("cv.pdf")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Analyser le CV" }));

    await waitFor(() => expect(analyze).toHaveBeenCalledOnce());
    expect(analyze.mock.calls[0]?.[0]).toMatchObject({
      job_offer: "Une offre",
      file_path: "/tmp/cv.pdf",
    });
    expect(analyze.mock.calls[0]?.[0].generation_id).toEqual(expect.any(String));
    expect(
      await screen.findByText("Analysé en 18,4 s · 1 024 tokens"),
    ).toBeInTheDocument();
  });

  it("masque le formulaire pendant l'analyse puis le restaure après un arrêt réel", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({
      path: "/tmp/cv.pdf",
      name: "cv.pdf",
    });
    let resolveAnalysis: ((value: ReturnType<typeof aiExecution<{
      resume: { resume: string; experiences: never[]; skills: never[]; education: never[] };
      job_offer: { title: string; skills: never[]; soft_skills: never[]; experience: null; keywords: never[] };
      score: { total: number; skills: null; experience: null; ats: null; present: never[]; missing: never[] };
      analysis: { recap: string; recommendations: never[]; content_recommendations: never[] };
    }>>) => void) | undefined;
    vi.spyOn(aiService, "analyzeResume").mockReturnValue(
      new Promise((resolve) => { resolveAnalysis = resolve; }),
    );
    let resolveCancel: (() => void) | undefined;
    const cancel = vi.spyOn(aiService, "cancel").mockReturnValue(
      new Promise((resolve) => { resolveCancel = resolve; }),
    );

    render(<ResumeAnalysisPage />, { wrapper });
    await userEvent.type(screen.getByLabelText(/Offre ciblée/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: "Choisir un fichier" }));
    await userEvent.click(screen.getByRole("button", { name: "Analyser le CV" }));

    expect(screen.queryByLabelText(/Offre ciblée/)).not.toBeInTheDocument();
    expect(screen.queryByText("cv.pdf")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Analyser le CV" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Arrêter" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Arrêter" }));
    expect(cancel).toHaveBeenCalledOnce();
    expect(screen.getByRole("button", { name: "Arrêt…" })).toBeDisabled();
    expect(screen.queryByText("Préparation du traitement…")).not.toBeInTheDocument();

    await act(async () => {
      resolveCancel?.();
      await Promise.resolve();
    });
    expect(await screen.findByLabelText(/Offre ciblée/)).toHaveValue("Une offre");
    expect(screen.getByText("cv.pdf")).toBeInTheDocument();

    await act(async () => {
      resolveAnalysis?.(aiExecution({
        resume: { resume: "Résultat tardif", experiences: [], skills: [], education: [] },
        job_offer: { title: "Dev", skills: [], soft_skills: [], experience: null, keywords: [] },
        score: { total: 99, skills: null, experience: null, ats: null, present: [], missing: [] },
        analysis: { recap: "Analyse tardive", recommendations: [], content_recommendations: [] },
      }));
      await Promise.resolve();
    });
    expect(screen.queryByText("Analyse tardive")).not.toBeInTheDocument();
  });

  it("restaure le formulaire et conserve le fichier après une erreur", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({ path: "/tmp/cv.pdf", name: "cv.pdf" });
    vi.spyOn(aiService, "analyzeResume").mockRejectedValue(
      new AppError({ code: "PROVIDER_ERROR", message: "Le fournisseur ne répond pas." }),
    );

    render(<ResumeAnalysisPage />, { wrapper });
    await userEvent.type(screen.getByLabelText(/Offre ciblée/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: "Choisir un fichier" }));
    await userEvent.click(screen.getByRole("button", { name: "Analyser le CV" }));

    expect(await screen.findByText("Le fournisseur ne répond pas.")).toBeInTheDocument();
    expect(screen.getByText("cv.pdf")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Analyser le CV" })).toBeInTheDocument();
  });
});

describe("échecs d'enregistrement", () => {
  /// Les mutations d'enregistrement n'avaient pas de `onError` : un refus du service Rust
  /// (nom trop long, contenu invalide, erreur SQLite) laissait l'écran strictement
  /// inchangé, et l'utilisateur croyait son document enregistré alors qu'il était perdu.
  it("signale le refus de duplication d'une version de CV", async () => {
    vi.spyOn(documentsService, "listResume").mockResolvedValue([]);
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-1", name: "CV Produit", created_at: "2026-08-30T00:00:00Z" }],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-1",
      name: "CV Produit",
      content: {
        resume: { resume: "", experiences: [], skills: [], education: [] },
        analysis: { recap: "", recommendations: [], content_recommendations: [] },
        job_offer: { title: "Dev", skills: [], soft_skills: [], experience: null, keywords: [] },
        profile_score: {
          total: 70,
          skills: null,
          experience: null,
          ats: null,
          present: [],
          missing: [],
        },
      },
      created_at: "2026-08-30T00:00:00Z",
    });
    vi.spyOn(documentsService, "saveResume").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Le contenu du CV est illisible" }),
    );

    render(<ResumeLibraryPage />, { wrapper });
    await waitFor(() => expect(screen.getByRole("button", { name: /Dupliquer/ })).toBeEnabled());
    await userEvent.click(screen.getByRole("button", { name: /Dupliquer/ }));

    await waitFor(() =>
      expect(useUiStore.getState().toasts.map((toast) => toast.title)).toContain(
        "Duplication impossible",
      ),
    );
  });

  it("signale le refus d'enregistrement d'une lettre", async () => {
    vi.spyOn(aiService, "generateCoverLetter").mockResolvedValue(aiExecution("Madame, Monsieur,"));
    vi.spyOn(documentsService, "saveCoverLetter").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Le nom de la lettre est trop long" }),
    );

    render(<LetterWriterPage />, { wrapper });

    const contexte = screen.getByLabelText("Contexte ou offre");
    await userEvent.type(contexte, "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /Rédiger la lettre/ }));

    await waitFor(() => expect(screen.getByRole("button", { name: /Enregistrer/ })).toBeEnabled());
    await userEvent.click(screen.getByRole("button", { name: /Enregistrer/ }));

    await waitFor(() =>
      expect(useUiStore.getState().toasts.map((toast) => toast.title)).toContain(
        "Enregistrement impossible",
      ),
    );
  });
});

describe("collage d'une offre depuis le presse-papiers", () => {
  it("remplit le champ avec le contenu lu côté natif", async () => {
    vi.spyOn(documentsService, "readClipboard").mockResolvedValue(
      "Administrateur Système et Réseau chez Astek",
    );

    render(<LetterWriterPage />, { wrapper });
    await userEvent.click(screen.getByRole("button", { name: "Coller" }));

    await waitFor(() =>
      expect(screen.getByLabelText("Contexte ou offre")).toHaveValue(
        "Administrateur Système et Réseau chez Astek",
      ),
    );
  });

  it("prévient quand le presse-papiers est inaccessible au lieu de rester muet", async () => {
    vi.spyOn(documentsService, "readClipboard").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Le presse-papiers ne contient pas de texte." }),
    );

    render(<LetterWriterPage />, { wrapper });
    await userEvent.click(screen.getByRole("button", { name: "Coller" }));

    await waitFor(() =>
      expect(useUiStore.getState().toasts.map((toast) => toast.title)).toContain(
        "Collage impossible",
      ),
    );
  });
});

describe("retouche de la lettre sur la page", () => {
  async function lettreGeneree(contenu = "Madame, Monsieur,") {
    vi.spyOn(aiService, "generateCoverLetter").mockResolvedValue(aiExecution(contenu));
    const save = vi.spyOn(documentsService, "saveCoverLetter").mockResolvedValue({
      id: "lettre-1",
      name: "Lettre — Candidature",
      company: null,
      job_title: null,
      recipient: null,
      recipient_address: null,
      job_reference: null,
      tone: "formal",
      length: "medium",
      content: contenu,
      created_at: "2026-08-30T00:00:00Z",
    });
    render(<LetterWriterPage />, { wrapper });
    await userEvent.type(screen.getByLabelText("Contexte ou offre"), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /Rédiger la lettre/ }));
    const corps = await screen.findByLabelText("Contenu de la lettre");
    await waitFor(() => expect(corps).toHaveTextContent("Madame, Monsieur,"));
    return { corps, save };
  }

  it("enregistre le texte tel qu'il a été modifié dans l'aperçu", async () => {
    const { corps, save } = await lettreGeneree();

    await userEvent.type(corps, " Astek");
    await userEvent.click(screen.getByRole("button", { name: /Enregistrer/ }));

    await waitFor(() =>
      expect(save.mock.lastCall?.[0].content).toContain("Astek"),
    );
  });

  it("porte l'alignement demandé jusqu'au contenu enregistré", async () => {
    const { corps, save } = await lettreGeneree();

    await userEvent.click(corps);
    await userEvent.click(screen.getByRole("button", { name: "Centrer" }));
    await userEvent.click(screen.getByRole("button", { name: /Enregistrer/ }));

    await waitFor(() =>
      expect(save.mock.lastCall?.[0].content).toBe('<p align="center">Madame, Monsieur,</p>'),
    );
  });

  it("porte la taille de texte demandée jusqu'au contenu enregistré", async () => {
    const { corps, save } = await lettreGeneree();

    await userEvent.click(corps);
    await userEvent.selectOptions(screen.getByLabelText("Taille du texte"), "large");
    await userEvent.click(screen.getByRole("button", { name: /Enregistrer/ }));

    await waitFor(() =>
      expect(save.mock.lastCall?.[0].content).toBe('<p size="large">Madame, Monsieur,</p>'),
    );
  });
});

describe("itérations sur la lettre", () => {
  async function redigerUneLettre(contenu = "Madame, Monsieur,") {
    vi.spyOn(aiService, "generateCoverLetter").mockResolvedValue(aiExecution(contenu));
    render(<LetterWriterPage />, { wrapper });
    await userEvent.type(screen.getByLabelText("Contexte ou offre"), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /Rédiger la lettre/ }));
  }

  it("remplace le brief par les itérations et annonce la durée de rédaction", async () => {
    await redigerUneLettre();

    expect(
      await screen.findByText("Lettre rédigée en 18,4 s · 1 024 tokens"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Que faut-il changer ?")).toBeInTheDocument();
    expect(screen.queryByLabelText("Contexte ou offre")).not.toBeInTheDocument();
  });

  it("arrête la première rédaction et ignore son résultat tardif", async () => {
    let resolveGeneration: ((value: ReturnType<typeof aiExecution<string>>) => void) | undefined;
    vi.spyOn(aiService, "generateCoverLetter").mockReturnValue(
      new Promise((resolve) => { resolveGeneration = resolve; }),
    );
    let resolveCancel: (() => void) | undefined;
    const cancel = vi.spyOn(aiService, "cancel").mockReturnValue(
      new Promise((resolve) => { resolveCancel = resolve; }),
    );
    render(<LetterWriterPage />, { wrapper });
    await userEvent.type(screen.getByLabelText("Contexte ou offre"), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /Rédiger la lettre/ }));

    await userEvent.click(screen.getByRole("button", { name: "Arrêter" }));
    expect(cancel).toHaveBeenCalledOnce();
    expect(screen.getByRole("button", { name: "Arrêt…" })).toBeDisabled();
    expect(screen.queryByText("Préparation du traitement…")).not.toBeInTheDocument();

    await act(async () => {
      resolveCancel?.();
      await Promise.resolve();
    });
    expect(await screen.findByRole("button", { name: /Rédiger la lettre/ })).toBeInTheDocument();
    expect(screen.queryByText("Rédaction impossible")).not.toBeInTheDocument();

    await act(async () => {
      resolveGeneration?.(aiExecution("Réponse tardive"));
      await Promise.resolve();
    });
    expect(screen.queryByText("Réponse tardive")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Enregistrer/ })).not.toBeInTheDocument();
  });

  it("cumule les consignes successives dans la demande envoyée au modèle", async () => {
    await redigerUneLettre();
    const generate = vi.mocked(aiService.generateCoverLetter);

    await userEvent.type(await screen.findByLabelText("Que faut-il changer ?"), "Plus court");
    await userEvent.click(screen.getByRole("button", { name: /Régénérer avec cette consigne/ }));
    await waitFor(() => expect(screen.getByText(/Lettre régénérée en/)).toBeInTheDocument());

    await userEvent.type(screen.getByLabelText("Que faut-il changer ?"), "Plus formel");
    await userEvent.click(screen.getByRole("button", { name: /Régénérer avec cette consigne/ }));

    await waitFor(() =>
      expect(generate.mock.lastCall?.[0].instruction).toBe("Plus court ; Plus formel"),
    );
  });

  it("abandonne la lettre et rend le brief après confirmation", async () => {
    await redigerUneLettre();

    await userEvent.click(await screen.findByRole("button", { name: "Annuler" }));
    await userEvent.click(screen.getByRole("button", { name: "Abandonner" }));

    expect(screen.getByLabelText("Contexte ou offre")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Enregistrer/ })).not.toBeInTheDocument();
  });

  it("laisse rouvrir le brief pour changer le ton ou l'offre", async () => {
    await redigerUneLettre();

    await userEvent.click(await screen.findByRole("button", { name: "Revenir au brief" }));

    expect(screen.getByLabelText("Contexte ou offre")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Revenir aux itérations" })).toBeInTheDocument();
  });
});

describe("bibliothèque CV workspace", () => {
  function listerWorkspace(workspace: ResumeWorkspace) {
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-ws", name: "CV Workspace", created_at: "2026-08-30T00:00:00Z" }],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-ws",
      name: "CV Workspace",
      content: workspace,
      created_at: "2026-08-30T00:00:00Z",
    });
  }

  it("affiche ResumePaper, ouvre le workspace avec Modifier et exporte le document", async () => {
    const workspace = workspaceFixture({ profile: "Profil visible en bibliothèque." });
    listerWorkspace(workspace);
    const exportPdf = vi.spyOn(documentsService, "exportPdf").mockResolvedValue(true);

    render(<ResumeLibraryPage />, { wrapper });
    expect(await screen.findByText("Profil visible en bibliothèque.")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /Modifier/ }));
    expect(navigateMock).toHaveBeenCalledWith("/documents/generate-resume", {
      state: { workspace, name: "CV Workspace" },
    });

    await userEvent.click(screen.getByRole("button", { name: /Exporter PDF/ }));
    await waitFor(() =>
      expect(exportPdf).toHaveBeenCalledWith(workspace.document),
    );
  });

  it("prépare une génération historique seulement à l'export", async () => {
    const generation = {
      resume: { resume: "Résumé historique.", experiences: [], skills: [], education: [] },
      analysis: { recap: "", recommendations: [], content_recommendations: [] },
      job_offer: { title: "Dev", skills: [], soft_skills: [], experience: null, keywords: [] },
      profile_score: { total: 72, skills: null, experience: null, ats: null, present: [], missing: [] },
      recommendation_error: null,
    };
    const prepared = workspaceFixture({ profile: "Document préparé à l'export." });
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-old", name: "CV Historique", created_at: "2026-08-30T00:00:00Z" }],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-old",
      name: "CV Historique",
      content: generation,
      created_at: "2026-08-30T00:00:00Z",
    });
    const prepareResume = vi.spyOn(documentsService, "prepareResume").mockResolvedValue(prepared);
    const exportPdf = vi.spyOn(documentsService, "exportPdf").mockResolvedValue(true);
    const saveResume = vi.spyOn(documentsService, "saveResume");

    render(<ResumeLibraryPage />, { wrapper });
    expect(await screen.findByText("Résumé historique.")).toBeInTheDocument();
    expect(prepareResume).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: /Exporter PDF/ }));
    await waitFor(() => expect(prepareResume).toHaveBeenCalledWith(generation));
    await waitFor(() =>
      expect(exportPdf).toHaveBeenCalledWith(prepared.document),
    );
    expect(saveResume).not.toHaveBeenCalled();
  });
});

describe("décisions ATS et confirmation profil dans le générateur de CV", () => {
  function missingSkillWorkspace(): ResumeWorkspace {
    const workspace = workspaceFixture();
    return {
      ...workspace,
      score: { ...workspace.score, missing: ["Docker"] },
      proposals: [
        {
          id: "missing-skill-docker",
          kind: "missing_skill",
          target: { type: "skill_group", group_id: "group-1" },
          label: "Docker",
          original_text: null,
          proposed_text: "Docker",
          gain: 5,
          status: "pending",
          applicable: true,
        },
      ],
    };
  }

  it("signale une compétence absente du profil sans proposer de l'ajouter au CV", async () => {
    vi.spyOn(aiService, "generateResume").mockResolvedValue(aiExecution({
      resume: { resume: "", experiences: [], skills: [], education: [] },
      analysis: { recap: "", recommendations: [], content_recommendations: [] },
      job_offer: { title: "Développeur", skills: [], soft_skills: [], experience: null, keywords: [] },
      profile_score: { total: 60, skills: null, experience: null, ats: null, present: [], missing: [] },
      recommendation_error: null,
    }));
    const workspace = missingSkillWorkspace();
    vi.spyOn(documentsService, "prepareResume").mockResolvedValue(workspace);

    render(<ResumeGeneratorPage />, { wrapper });
    await userEvent.type(screen.getByLabelText(/Texte de l’offre/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /Générer le CV ciblé/ }));

    expect(await screen.findByText("Docker")).toBeInTheDocument();
    expect(screen.getByText(/absentes de votre profil/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Accepter Docker" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Ajouter au profil" })).not.toBeInTheDocument();
  });

  it("efface l'offre une fois le CV généré, et sait y revenir", async () => {
    vi.spyOn(aiService, "generateResume").mockResolvedValue(aiExecution({
      resume: { resume: "", experiences: [], skills: [], education: [] },
      analysis: { recap: "", recommendations: [], content_recommendations: [] },
      job_offer: { title: "Développeur", skills: [], soft_skills: [], experience: null, keywords: [] },
      profile_score: { total: 60, skills: null, experience: null, ats: null, present: [], missing: [] },
      recommendation_error: null,
    }));
    vi.spyOn(documentsService, "prepareResume").mockResolvedValue(missingSkillWorkspace());

    render(<ResumeGeneratorPage />, { wrapper });
    await userEvent.type(screen.getByLabelText(/Texte de l’offre/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /Générer le CV ciblé/ }));

    // L'offre laisse la place à l'aperçu : sans cela le papier A4 restait comprimé.
    const revenir = await screen.findByRole("button", { name: /Modifier l’offre/ });
    expect(screen.queryByLabelText(/Texte de l’offre/)).not.toBeInTheDocument();

    await userEvent.click(revenir);
    expect(screen.getByLabelText(/Texte de l’offre/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /Revenir au CV/ }));
    expect(screen.queryByLabelText(/Texte de l’offre/)).not.toBeInTheDocument();
  });
});
