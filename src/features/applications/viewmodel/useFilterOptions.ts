import { useCallback } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  ApplicationTypes,
  CompanySizes,
  WeeklyWorkSchedules,
  useReferentials,
} from "@/features/referentials";
import { COMPANIES_KEY, useCompany, useCompanySearch } from "@/features/companies";
import type { GlyphTone } from "@/shared/ui";
import { Statuses } from "../model/statuses";
import { Channels } from "../model/presentation";
import type { FilterKey, ListKey } from "../model/filterFields";

/** Valeur proposée au second niveau du menu de filtre. */
export interface FilterOption {
  readonly value: string;
  readonly label: string;
  readonly glyph?: GlyphTone;
}

/**
 * Options et libellés du menu de filtre. Les valeurs des référentiels viennent de la base,
 * jamais d'une liste écrite ici : une seconde copie divergerait au premier ajout.
 */
export function useFilterOptions(companyId: string | null) {
  const referentials = useReferentials().data;
  const company = useCompany(companyId);

  const options = useCallback(
    (key: ListKey): readonly FilterOption[] => {
      switch (key) {
        case "status":
          return Statuses.map((status) => ({ value: status.value, label: status.label, glyph: status.glyph }));
        case "channel":
          return Channels.map((channel) => ({ value: channel.value, label: channel.long }));
        case "contract":
          return referentials.contract_types.map((item) => ({ value: item.code, label: item.name }));
        case "application_type":
          return ApplicationTypes.map((item) => ({ value: item.value, label: item.label }));
        case "domain":
          return referentials.professional_domains.map((item) => ({ value: item.code, label: item.name }));
        case "company_type":
          return referentials.company_types.map((item) => ({ value: item.code, label: item.name }));
        case "sector":
          return referentials.sectors.map((item) => ({ value: item.id, label: item.name }));
        case "size":
          return CompanySizes.map((item) => ({ value: item.value, label: item.label }));
        case "schedule":
          return WeeklyWorkSchedules.map((item) => ({ value: item.value, label: item.label }));
      }
    },
    [referentials],
  );

  const label = useCallback(
    (key: FilterKey, value: string): string => {
      if (key === "company") return company.data?.name ?? "…";
      if (key === "job_title" || key === "city" || key === "sent" || key === "hours") return value;
      return options(key).find((option) => option.value === value)?.label ?? value;
    },
    [company.data, options],
  );

  return { options, label };
}

/** Entreprises proposées au second niveau de « Entreprise », filtrées en base. */
export function useCompanyFilterOptions(search: string, enabled: boolean) {
  const fetchPage = useCompanySearch();
  return useQuery({
    queryKey: [...COMPANIES_KEY, "filtre", search],
    queryFn: () => fetchPage({ page: 1, page_size: 12, search }),
    enabled,
  });
}
