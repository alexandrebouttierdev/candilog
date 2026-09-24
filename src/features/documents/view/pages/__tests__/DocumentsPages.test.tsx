import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { documentsService } from "../../../services/documentsService";
import { workspaceFixture } from "../../../model/resumeWorkspace";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { useUiStore } from "@/shared/lib/ui-store";
import { DocumentsPage } from "../DocumentsPage";
import { ResumeGeneratorPage } from "../ResumeGeneratorPage";
import { ResumeAnalysisPage } from "../ResumeAnalysisPage";
import { LetterWriterPage } from "../LettersPages";
import { AppError } from "@/shared/types/app-error";
import {
  aiService,
  type AiExecution,
  type ImportedResumeAnalysis,
  useAiOperationStore,
} from "@/features/ai";

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

function emptyJobOffer(title: string) {
  return {
    title,
    skills: [],
    soft_skills: [],
    experience: null,
    keywords: [],
    requirements: [],
    location: null,
  };
}

function emptyMatchScore(total: number) {
  return {
    total,
    skills: null,
    experience: null,
    ats: null,
    present: [],
    missing: [],
    breakdown: [],
    evaluations: [],
    critical_requirements_penalty: 0,
  };
}

/** Bibliothèque sans lettre : les décomptes des onglets interrogent aussi les lettres. */
function sansLettres() {
  vi.spyOn(documentsService, "listCoverLettersPage").mockResolvedValue({
    items: [],
    total: 0,
    page: 1,
    page_size: 50,
    total_pages: 1,
  });
}

beforeEach(() => {
  vi.restoreAllMocks();
  useAiOperationStore.setState({ active: null });
  navigateMock.mockReset();
  useUiStore.setState({ toasts: [] });
});

describe("bibliothèque de documents", () => {
  it("recherche et charge la suite côté backend", async () => {
    sansLettres();
    const paged = vi.spyOn(documentsService, "listResumePage").mockImplementation(({ page, page_size, search }) =>
      Promise.resolve({
        items: [{ id: `${search || "cv"}-1`, name: `${search || "CV"} 1`, created_at: "2026-08-30T00:00:00Z", ats_score: null, target_title: null }],
        total: search ? 1 : 60,
        page,
        page_size,
        total_pages: 1,
      }),
    );
    vi.spyOn(documentsService, "getResume").mockImplementation((id) => Promise.resolve({ id, name: id, content: null, created_at: "2026-08-30T00:00:00Z" }));

    render(<DocumentsPage filter="resumes" />, { wrapper });
    await waitFor(() =>
      expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 50, search: "", scored_only: false }),
    );

    await userEvent.click(await screen.findByRole("button", { name: /Afficher plus/ }));
    await waitFor(() =>
      expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 100, search: "", scored_only: false }),
    );

    await userEvent.type(screen.getByRole("searchbox", { name: "Rechercher un document" }), "cible");
    await waitFor(() =>
      expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 50, search: "cible", scored_only: false }),
    );
    // La commande exhaustive n'existe plus : la pagination en base est le seul chemin de
    // chargement de la bibliothèque, et `commandes-ipc.test.ts` verrouille l'inventaire.
    expect(documentsService).not.toHaveProperty("listResume");
  });

  it("ne demande que les CV analysés dans l'onglet Analyses", async () => {
    sansLettres();
    const paged = vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-1", name: "CV ciblé", created_at: "2026-08-30T00:00:00Z", ats_score: 82, target_title: "Chargé d'exploitation" }],
      total: 1,
      page: 1,
      page_size: 50,
      total_pages: 1,
    });
    vi.spyOn(documentsService, "getResume").mockResolvedValue({ id: "cv-1", name: "CV ciblé", content: null, created_at: "2026-08-30T00:00:00Z" });

    render(<DocumentsPage filter="analyses" />, { wrapper });

    expect(await screen.findByText("ATS 82")).toBeInTheDocument();
    expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 50, search: "", scored_only: true });
  });
});

describe("analyse explicite d'un CV sélectionné", () => {
  it("sélectionne le PDF sans analyser puis lance l'analyse au clic dédié", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({ name: "cv.pdf" });
    const analyze = vi.spyOn(aiService, "analyzeResume").mockResolvedValue({
      output: {
        resume: { resume: "Profil", experiences: [], skills: [], education: [] },
        job_offer: emptyJobOffer("Développeur"),
        score: {
          ...emptyMatchScore(72),
          missing: ["Java"],
          breakdown: [
            { category: "occupation" as const, label: "Métier / fonction", score: 80, weight: 60 },
          ],
        },
        analysis: { recap: "Analyse terminée", recommendations: [], content_recommendations: [] },
        method_used: "text" as const,
        fallback_used: false,
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
    // Le chemin ne traverse plus l'IPC : le fichier analysé est celui que Rust a retenu
    // au moment du choix dans le dialogue natif.
    expect(analyze.mock.calls[0]?.[0]).toMatchObject({ job_offer: "Une offre" });
    expect(analyze.mock.calls[0]?.[0]).not.toHaveProperty("file_path");
    expect(analyze.mock.calls[0]?.[0].generation_id).toEqual(expect.any(String));
    expect(
      await screen.findByText("Analysé en 18,4 s · 1 024 tokens"),
    ).toBeInTheDocument();
    expect(screen.getByText("Métier / fonction")).toBeInTheDocument();
    expect(screen.getByText("Exigence absente : Java")).toBeInTheDocument();
    expect(screen.queryByText("À appliquer dans l’éditeur de CV")).not.toBeInTheDocument();
  });

  it("masque le formulaire pendant l'analyse puis le restaure après un arrêt réel", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({ name: "cv.pdf" });
    let resolveAnalysis: ((value: AiExecution<ImportedResumeAnalysis>) => void) | undefined;
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
        job_offer: emptyJobOffer("Dev"),
        score: emptyMatchScore(99),
        analysis: { recap: "Analyse tardive", recommendations: [], content_recommendations: [] },
        method_used: "text" as const,
        fallback_used: false,
      }));
      await Promise.resolve();
    });
    expect(screen.queryByText("Analyse tardive")).not.toBeInTheDocument();
  });

  it("restaure le formulaire et conserve le fichier après une erreur", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({ name: "cv.pdf" });
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
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-1", name: "CV Produit", created_at: "2026-08-30T00:00:00Z", ats_score: 70, target_title: "Dev" }],
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
        job_offer: emptyJobOffer("Dev"),
        profile_score: emptyMatchScore(70),
      },
      created_at: "2026-08-30T00:00:00Z",
    });
    vi.spyOn(documentsService, "saveResume").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Le contenu du CV est illisible" }),
    );

    sansLettres();
    render(<DocumentsPage filter="resumes" />, { wrapper });
    await waitFor(() => expect(screen.getByRole("button", { name: "PDF" })).toBeEnabled());
    await userEvent.click(screen.getByRole("button", { name: "Autres actions" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Dupliquer" }));

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
    await waitFor(() => expect(corps).toHaveTextContent(contenu));
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

  it("corrige l'orthographe à la demande en conservant la lettre éditable", async () => {
    await lettreGeneree("Je suis motive.");
    const correction = vi.spyOn(aiService, "correctFrench").mockResolvedValue(aiExecution({
      fields: [{ id: "paragraph:0:run:0", text: "Je suis motivé." }],
    }));

    await userEvent.click(screen.getByRole("button", { name: "Corriger l’orthographe" }));

    await waitFor(() => expect(correction).toHaveBeenCalledOnce());
    expect(await screen.findByLabelText("Contenu de la lettre")).toHaveTextContent("Je suis motivé.");
    expect(useUiStore.getState().toasts.map((toast) => toast.title)).toContain("Orthographe corrigée");
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

  it("ouvre les corrections et annonce la durée de rédaction", async () => {
    await redigerUneLettre();

    expect(
      await screen.findByText("Lettre rédigée en 18,4 s · 1 024 tokens"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Que faut-il changer ?")).toBeInTheDocument();
    // Le brief reste visible : changer le ton ou l'offre ne demande aucun aller-retour.
    expect(screen.getByLabelText("Contexte ou offre")).toBeInTheDocument();
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

  it("cumule les consignes successives et renvoie la lettre précédente au modèle", async () => {
    await redigerUneLettre("Madame, Monsieur, première version.");
    const generate = vi.mocked(aiService.generateCoverLetter);

    await userEvent.type(await screen.findByLabelText("Que faut-il changer ?"), "Plus court");
    await userEvent.click(screen.getByRole("button", { name: "Envoyer" }));
    await waitFor(() => expect(screen.getByText(/Lettre régénérée en/)).toBeInTheDocument());
    await waitFor(() =>
      expect(generate.mock.lastCall?.[0]).toMatchObject({
        instruction: "Plus court",
        previous_cover_letter: "Madame, Monsieur, première version.",
      }),
    );

    // Consigne rapide : envoyée telle quelle, cumulée avec la précédente.
    await userEvent.click(await screen.findByRole("button", { name: "Moins formel" }));

    await waitFor(() =>
      expect(generate.mock.lastCall?.[0].instruction).toBe("Plus court ; Moins formel"),
    );
    expect(generate.mock.lastCall?.[0].previous_cover_letter).toBeTruthy();
  });

  it("abandonne la lettre et rend le brief après confirmation", async () => {
    await redigerUneLettre();

    await userEvent.click(await screen.findByRole("button", { name: "Abandonner…" }));
    await userEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Abandonner" }));

    expect(screen.getByLabelText("Contexte ou offre")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Rédiger la lettre/ })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Enregistrer/ })).not.toBeInTheDocument();
  });

  it("relance une nouvelle lettre avec le ton changé, sans quitter l'écran", async () => {
    await redigerUneLettre();
    const generate = vi.mocked(aiService.generateCoverLetter);

    await userEvent.click(await screen.findByRole("radio", { name: "Naturel" }));
    await userEvent.click(screen.getByRole("button", { name: "Rédiger une nouvelle lettre" }));

    await waitFor(() => expect(generate.mock.lastCall?.[0]).toMatchObject({ tone: "casual" }));
  });
});

describe("bibliothèque CV workspace", () => {
  function listerWorkspace(workspace: ResumeWorkspace) {
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-ws", name: "CV Workspace", created_at: "2026-08-30T00:00:00Z", ats_score: null, target_title: null }],
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

  it("ouvre le workspace dans l'éditeur et exporte le document", async () => {
    const workspace = workspaceFixture({ profile: "Profil visible en bibliothèque." });
    listerWorkspace(workspace);
    sansLettres();
    const exportPdf = vi.spyOn(documentsService, "exportPdf").mockResolvedValue(true);

    render(<DocumentsPage filter="resumes" />, { wrapper });
    await waitFor(() => expect(screen.getByRole("button", { name: "Ouvrir" })).toBeEnabled());

    await userEvent.click(screen.getByRole("button", { name: "Ouvrir" }));
    expect(navigateMock).toHaveBeenCalledWith("/documents/generate-resume", {
      state: { workspace, name: "CV Workspace" },
    });

    await userEvent.click(screen.getByRole("button", { name: "PDF" }));
    await waitFor(() =>
      expect(exportPdf).toHaveBeenCalledWith(workspace.document),
    );
  });

  it("prépare une génération historique seulement à l'export", async () => {
    const generation = {
      resume: { resume: "Résumé historique.", experiences: [], skills: [], education: [] },
      analysis: { recap: "", recommendations: [], content_recommendations: [] },
      job_offer: emptyJobOffer("Dev"),
      profile_score: emptyMatchScore(72),
      recommendation_error: null,
    };
    const prepared = workspaceFixture({ profile: "Document préparé à l'export." });
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-old", name: "CV Historique", created_at: "2026-08-30T00:00:00Z", ats_score: 72, target_title: "Dev" }],
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

    sansLettres();
    render(<DocumentsPage filter="resumes" />, { wrapper });
    await waitFor(() => expect(screen.getByRole("button", { name: "PDF" })).toBeEnabled());
    expect(prepareResume).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: "PDF" }));
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
      job_offer: emptyJobOffer("Développeur"),
      profile_score: emptyMatchScore(60),
      recommendation_error: null,
    }));
    const workspace = missingSkillWorkspace();
    vi.spyOn(documentsService, "prepareResume").mockResolvedValue(workspace);

    render(<ResumeGeneratorPage />, { wrapper });
    await userEvent.type(screen.getByLabelText(/Texte de l’offre/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /^Générer/ }));

    expect(await screen.findByText("Docker")).toBeInTheDocument();
    expect(screen.getByRole("dialog", { name: "Générer un CV" })).toBeInTheDocument();
    expect(screen.queryByText("Analysez une offre, générez un CV ciblé, exportez en PDF")).not.toBeInTheDocument();
    expect(screen.getByText(/absentes de votre profil/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Accepter Docker" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Ajouter au profil" })).not.toBeInTheDocument();
  });

  it("efface l'offre une fois le CV généré, et sait y revenir", async () => {
    vi.spyOn(aiService, "generateResume").mockResolvedValue(aiExecution({
      resume: { resume: "", experiences: [], skills: [], education: [] },
      analysis: { recap: "", recommendations: [], content_recommendations: [] },
      job_offer: emptyJobOffer("Développeur"),
      profile_score: emptyMatchScore(60),
      recommendation_error: null,
    }));
    vi.spyOn(documentsService, "prepareResume").mockResolvedValue(missingSkillWorkspace());

    render(<ResumeGeneratorPage />, { wrapper });
    await userEvent.type(screen.getByLabelText(/Texte de l’offre/), "Une offre");
    await userEvent.click(screen.getByRole("button", { name: /^Générer/ }));

    // L'offre laisse la place à l'aperçu : sans cela le papier A4 restait comprimé.
    const revenir = await screen.findByRole("button", { name: /Modifier l’offre/ });
    expect(screen.queryByLabelText(/Texte de l’offre/)).not.toBeInTheDocument();

    await userEvent.click(revenir);
    expect(screen.getByLabelText(/Texte de l’offre/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /Revenir au CV/ }));
    expect(screen.queryByLabelText(/Texte de l’offre/)).not.toBeInTheDocument();
  });
});
