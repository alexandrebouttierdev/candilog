import { useCallback, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { COMPANIES_KEY, companyService } from "@/features/companies";
import type { Company, CompanyFilter, NewCompany, RelationState } from "@/features/companies";
import { CONTACTS_KEY, contactService } from "@/features/contacts";
import type { Contact, NewContact } from "@/features/contacts";
import { APPLICATIONS_KEY, EMPTY_FILTER, applicationService } from "@/features/applications";
import type { ApplicationFilter } from "@/features/applications";
import type { CompanySize } from "@/shared/types/generated/companies";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export type RelationKind = "companies" | "contacts";

/** Lignes chargées par groupe, puis à chaque « Afficher plus ». */
export const RELATION_STEP = 50;

/** Groupes de l'écran Relations (`screens/05-companies.png`, `06-network.png`). */
export const COMPANY_GROUPS: ReadonlyArray<{
  key: RelationState;
  label: string;
  note: string;
  glyph: "g" | "n" | "c";
}> = [
  { key: "active", label: "En cours", note: "au moins une candidature ouverte", glyph: "g" },
  { key: "watch", label: "Repérées", note: "aucune candidature envoyée", glyph: "n" },
  { key: "closed", label: "Clôturées", note: "toutes les candidatures refusées", glyph: "c" },
];

export const CONTACT_GROUPS: ReadonlyArray<{
  key: "linked" | "network";
  label: string;
  note: string;
  glyph: "g" | "n";
}> = [
  { key: "linked", label: "Recruteurs et managers", note: "rattachés à une candidature", glyph: "g" },
  { key: "network", label: "Réseau", note: "contacts personnels et anciens collègues", glyph: "n" },
];

/** Critères fins conservés de la v1 : secteur, type et taille ; rôle pour les contacts. */
export interface RelationCriteria {
  readonly sector_id: string | null;
  readonly company_type_id: string | null;
  readonly company_size: CompanySize | null;
  readonly tracking_role: string | null;
}

export const EMPTY_CRITERIA: RelationCriteria = {
  sector_id: null,
  company_type_id: null,
  company_size: null,
  tracking_role: null,
};

function applicationsOf(filter: Partial<ApplicationFilter>): ApplicationFilter {
  return { ...EMPTY_FILTER, search: "", sort: "date", descending: true, ids: [], ...filter };
}

/**
 * Orchestration de l'écran Relations : entreprises et contacts sur un même écran, chaque
 * population groupée (En cours / Repérées / Clôturées ; Recruteurs et managers / Réseau).
 *
 * Chaque groupe est une requête SQLite séparée, filtrée **avant** la limite : grouper en
 * mémoire une page déjà coupée afficherait des groupes tronqués sans le dire.
 */
export function useRelationsViewModel(kind: RelationKind) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [search, setSearchState] = useState("");
  const searchQuery = useDebounce(search);
  const [criteria, setCriteriaState] = useState<RelationCriteria>(EMPTY_CRITERIA);
  const [limits, setLimits] = useState<Record<string, number>>({});
  // `?id=` ouvre une fiche précise : l'entreprise d'un contact, une fiche citée ailleurs.
  const [searchParams] = useSearchParams();
  const [selectedId, setSelectedId] = useState<string | null>(() => searchParams.get("id"));

  const limitOf = (key: string) => limits[key] ?? RELATION_STEP;

  const companyFilter = (relation_state: RelationState): CompanyFilter => ({
    search: searchQuery,
    sector_id: criteria.sector_id,
    company_type_id: criteria.company_type_id,
    company_size: criteria.company_size,
    relation_state,
  });

  const companyQueries = useQueries({
    queries: COMPANY_GROUPS.map((group) => ({
      queryKey: [...COMPANIES_KEY, "relations", group.key, { search: searchQuery, criteria, limit: limitOf(group.key) }],
      queryFn: () =>
        companyService.listPage({ page: 1, page_size: limitOf(group.key), filter: companyFilter(group.key) }),
      enabled: kind === "companies",
    })),
  });

  const contactQueries = useQueries({
    queries: CONTACT_GROUPS.map((group) => ({
      queryKey: [...CONTACTS_KEY, "relations", group.key, { search: searchQuery, role: criteria.tracking_role, limit: limitOf(group.key) }],
      queryFn: () =>
        contactService.listPage({
          page: 1,
          page_size: limitOf(group.key),
          search: searchQuery,
          tracking_role: criteria.tracking_role,
          linked: group.key === "linked",
        }),
      enabled: kind === "contacts",
    })),
  });

  // Décomptes de la bascule : totaux sans critère, toujours chargés pour les deux onglets.
  const companyCount = useQuery({
    queryKey: [...COMPANIES_KEY, "navigation"],
    queryFn: () =>
      companyService.listPage({
        page: 1,
        page_size: 1,
        filter: { search: "", sector_id: null, company_type_id: null, company_size: null, relation_state: null },
      }),
  });
  const contactCount = useQuery({
    queryKey: [...CONTACTS_KEY, "navigation"],
    queryFn: () => contactService.listPage({ page: 1, page_size: 1, search: "", tracking_role: null }),
  });

  const companyGroups = COMPANY_GROUPS.map((group, index) => ({
    ...group,
    items: companyQueries[index]?.data?.items ?? [],
    total: companyQueries[index]?.data?.total ?? 0,
  }));
  const contactGroups = CONTACT_GROUPS.map((group, index) => ({
    ...group,
    items: contactQueries[index]?.data?.items ?? [],
    total: contactQueries[index]?.data?.total ?? 0,
  }));

  const companies = companyGroups.flatMap((group) => group.items);
  const contacts = contactGroups.flatMap((group) => group.items);
  // Comme la maquette, la colonne de droite n'est jamais vide : à défaut de sélection
  // explicite, la première fiche est ouverte.
  const company: Company | null =
    kind === "companies" ? (companies.find((item) => item.id === selectedId) ?? companies[0] ?? null) : null;
  const contact: Contact | null =
    kind === "contacts" ? (contacts.find((item) => item.id === selectedId) ?? contacts[0] ?? null) : null;

  const linkedFilter = company
    ? applicationsOf({ company_id: company.id })
    : contact
      ? applicationsOf({ contact_id: contact.id })
      : null;
  const linked = useQuery({
    queryKey: [...APPLICATIONS_KEY, "relations", linkedFilter],
    queryFn: () => applicationService.listPage({ page: 1, page_size: 20, filter: linkedFilter ?? applicationsOf({}) }),
    enabled: linkedFilter !== null,
  });

  const queries = kind === "companies" ? companyQueries : contactQueries;

  const invalidate = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: COMPANIES_KEY });
    await queryClient.invalidateQueries({ queryKey: CONTACTS_KEY });
  }, [queryClient]);

  const reportFailure = (title: string) => (error: unknown) =>
    notify({ tone: "error", title, detail: error instanceof AppError ? error.message : undefined });

  const saveCompany = useMutation({
    mutationFn: (params: { id: string | null; input: NewCompany }) =>
      params.id ? companyService.update(params.id, params.input) : companyService.create(params.input),
    onSuccess: async (saved, params) => {
      await invalidate();
      setSelectedId(saved.id);
      notify({ tone: "success", title: params.id ? "Entreprise modifiée" : "Entreprise enregistrée", detail: saved.name });
    },
    onError: reportFailure("Enregistrement impossible"),
  });

  const saveContact = useMutation({
    mutationFn: (params: { id: string | null; input: NewContact }) =>
      params.id ? contactService.update(params.id, params.input) : contactService.create(params.input),
    onSuccess: async (saved, params) => {
      await invalidate();
      setSelectedId(saved.id);
      notify({
        tone: "success",
        title: params.id ? "Contact modifié" : "Contact enregistré",
        detail: `${saved.first_name} ${saved.name}`,
      });
    },
    onError: reportFailure("Enregistrement impossible"),
  });

  const remove = useMutation({
    mutationFn: (params: { kind: RelationKind; id: string }) =>
      params.kind === "companies" ? companyService.delete(params.id) : contactService.delete(params.id),
    onSuccess: async (_result, params) => {
      await invalidate();
      if (selectedId === params.id) setSelectedId(null);
      notify({ tone: "success", title: params.kind === "companies" ? "Entreprise supprimée" : "Contact supprimé" });
    },
    onError: reportFailure("Suppression impossible"),
  });

  const setSearch = useCallback((value: string) => {
    setSearchState(value);
    setLimits({});
  }, []);
  const setCriteria = useCallback((value: RelationCriteria) => {
    setCriteriaState(value);
    setLimits({});
  }, []);

  const activeCriteria =
    kind === "companies"
      ? [criteria.sector_id, criteria.company_type_id, criteria.company_size].filter(Boolean).length
      : criteria.tracking_role
        ? 1
        : 0;

  return {
    kind,
    search,
    setSearch,
    criteria,
    setCriteria,
    activeCriteria,
    companyGroups,
    contactGroups,
    counts: { companies: companyCount.data?.total ?? null, contacts: contactCount.data?.total ?? null },
    total: (kind === "companies" ? companyGroups : contactGroups).reduce((sum, group) => sum + group.total, 0),
    company,
    contact,
    linkedApplications: linked.data?.items ?? [],
    linkedTotal: linked.data?.total ?? 0,
    isLoading: queries.some((query) => query.isPending),
    error: queries.find((query) => query.error)?.error ?? null,
    reload: () => void invalidate(),
    select: setSelectedId,
    showMore: (key: string) => setLimits((current) => ({ ...current, [key]: limitOf(key) + RELATION_STEP })),
    saveCompany: saveCompany.mutateAsync,
    saveContact: saveContact.mutateAsync,
    isSaving: saveCompany.isPending || saveContact.isPending,
    remove: remove.mutateAsync,
    isDeleting: remove.isPending,
  };
}
