import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useUiStore } from "@/shared/lib/ui-store";
import type { Profile, ProfilePayload } from "@/shared/types/generated/profile";
import { profileService } from "../../services/profileService";
import { PROFILE_DERIVED_META, useProfileViewModel } from "../useProfileViewModel";

const identite: Profile["identity"] = {
  first_name: "Jean",
  name: "Rivière",
  email: "jean@exemple.fr",
  phone: null,
  address: null,
  city: null,
  title: null,
  resume: null,
  birth_date: null,
  age: null,
  availability: null,
  desired_contracts: null,
  linkedin: null,
  github: null,
  website: null,
};

const profil: Profile = {
  identity: identite,
  photo: null,
  experiences: [],
  skills: [],
  education: [],
  languages: [],
  projects: [],
  certifications: [],
  interests: [],
};

const payload: ProfilePayload = {
  profile: profil,
  completion: 40,
  incomplete_sections: [],
  updated_at: null,
};

/** Requête dérivée du profil, comme la composition du CV de base (`PROFILE_DERIVED_META`). */
const COMPOSITION_KEY = ["composition-derivee"];
/** Requête sans rapport avec le profil : aucune écriture du profil ne doit la périmer. */
const AUTRE_KEY = ["sans-rapport"];

function contexte() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  // Une composition déjà en cache, comme après une visite du CV de base.
  void client.prefetchQuery({
    queryKey: COMPOSITION_KEY,
    queryFn: () => ({ marqueur: "composition périmée" }),
    meta: PROFILE_DERIVED_META,
  });
  client.setQueryData(AUTRE_KEY, { marqueur: "intacte" });
  function wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  }
  return { client, wrapper };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(profileService, "load").mockResolvedValue(payload);
});

/**
 * Sans cette invalidation, revenir sur le CV de base dans les trente secondes suivant une
 * modification du profil affichait l'ancienne feuille (`staleTime` de 30 s), sous la mention
 * « composé depuis votre profil ».
 */
describe("invalidation de la composition du CV de base", () => {
  it("périme la composition quand le profil est enregistré", async () => {
    vi.spyOn(profileService, "save").mockResolvedValue(payload);
    const { client, wrapper } = contexte();
    const { result } = renderHook(() => useProfileViewModel(), { wrapper });

    await act(async () => { await result.current.save(profil); });

    await waitFor(() => expect(client.getQueryState(COMPOSITION_KEY)?.isInvalidated).toBe(true));
    expect(client.getQueryState(AUTRE_KEY)?.isInvalidated).toBe(false);
  });

  it("périme la composition quand un import est appliqué", async () => {
    vi.spyOn(profileService, "applyImport").mockResolvedValue({ added: 1, replaced: 0, skipped: 0 });
    const { client, wrapper } = contexte();
    const { result } = renderHook(() => useProfileViewModel(), { wrapper });

    await act(async () => {
      await result.current.applyImport({
        identity: [],
        experiences: [],
        skills: [],
        education: [],
        languages: [],
        projects: [],
        certifications: [],
        interests: [],
      });
    });

    await waitFor(() => expect(client.getQueryState(COMPOSITION_KEY)?.isInvalidated).toBe(true));
  });

  it("périme la composition quand le profil est réinitialisé", async () => {
    vi.spyOn(profileService, "reset").mockResolvedValue(payload);
    const { client, wrapper } = contexte();
    const { result } = renderHook(() => useProfileViewModel(), { wrapper });

    await act(async () => { await result.current.reset(); });

    await waitFor(() => expect(client.getQueryState(COMPOSITION_KEY)?.isInvalidated).toBe(true));
  });
});
