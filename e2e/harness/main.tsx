/**
 * Banc de rendu des feuilles de document, hors application.
 *
 * Il monte les **vrais** composants `ResumePaper` et `LetterPaper` avec les artefacts
 * produits par le scénario Rust de bout en bout (`src-tauri/tests/e2e_documents.rs`), sans
 * fenêtre Tauri ni IPC : c'est le seul moyen de mesurer la feuille telle que l'utilisateur
 * la voit, avec ses polices, sa densité recalculée et sa géométrie A4 réelle.
 *
 * Il ne fait partie ni de l'application ni de son bundle : `vite build` n'a qu'une entrée,
 * `index.html`, et ce fichier n'est jamais importé depuis `src/`.
 */
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { LetterContent } from "@/features/documents/view/components/LetterEditor";
import { LetterPaper } from "@/features/documents/view/components/LetterPaper";
import { ResumePaper } from "@/features/documents/view/components/ResumePaper";
import { PROFILE_KEY } from "@/features/profile/viewmodel/useProfileViewModel";
import type { Identity } from "@/shared/types/generated/profile";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import "@/styles.css";

type LettreArtefact = {
  identity: Identity;
  company: string | null;
  job_title: string | null;
  recipient: string | null;
  recipient_address: string | null;
  job_reference: string | null;
  content: string;
};

const parametres = new URLSearchParams(globalThis.location.search);
const dossier = parametres.get("dir") ?? "";
const genre = parametres.get("kind") ?? "resume";

/** Le banc annonce son état par cet attribut : Playwright attend `pret` avant de mesurer. */
function etat(valeur: string, detail?: string) {
  document.body.dataset["etat"] = valeur;
  if (detail !== undefined) document.body.dataset["detail"] = detail;
}

async function charger<T>(fichier: string): Promise<T> {
  const reponse = await fetch(`${dossier}/${fichier}`);
  if (!reponse.ok) throw new Error(`${fichier} : HTTP ${reponse.status}`);
  return (await reponse.json()) as T;
}

/**
 * `LetterPaper` lit l'identité par TanStack Query. Le banc pose la donnée dans le cache et
 * fige sa fraîcheur : sans cela le composant appellerait l'IPC, indisponible hors Tauri.
 */
function clientAvecProfil(identity: Identity): QueryClient {
  const client = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, gcTime: Infinity, retry: false } },
  });
  client.setQueryData(PROFILE_KEY, {
    profile: {
      identity,
      experiences: [],
      skills: [],
      education: [],
      languages: [],
      projects: [],
      certifications: [],
    },
    completion: 100,
    incomplete_sections: [],
    updated_at: null,
  });
  return client;
}

async function demarrer() {
  etat("chargement");
  const racine = createRoot(document.getElementById("banc") as HTMLElement);
  try {
    if (genre === "letter") {
      const lettre = await charger<LettreArtefact>("letter.json");
      racine.render(
        <StrictMode>
          <QueryClientProvider client={clientAvecProfil(lettre.identity)}>
            <LetterPaper
              editable={false}
              fields={{
                company: lettre.company,
                job_title: lettre.job_title,
                recipient: lettre.recipient,
                recipient_address: lettre.recipient_address,
                job_reference: lettre.job_reference,
              }}
            >
              <LetterContent content={lettre.content} />
            </LetterPaper>
          </QueryClientProvider>
        </StrictMode>,
      );
    } else {
      const workspace = await charger<ResumeWorkspace>("workspace.json");
      racine.render(
        <StrictMode>
          <ResumePaper workspace={workspace} editable={false} onChange={() => {}} />
        </StrictMode>,
      );
    }
  } catch (erreur) {
    etat("erreur", erreur instanceof Error ? erreur.message : String(erreur));
    return;
  }
  // Le papier recalcule sa densité après `document.fonts.ready` : mesurer avant donnerait
  // la géométrie d'un palier qui ne sera pas celui affiché.
  await document.fonts.ready;
  await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  etat("pret");
}

void demarrer();
