import { companyService } from "../services/companyService";
import type { Page } from "@/shared/types/page";
import type { EntityOption } from "@/shared/ui";

/** Page d'entreprises pour un EntityPicker — API publique de la feature. */
export async function fetchCompanyPickerPage(params: {
  page: number;
  page_size: number;
  search: string;
}): Promise<Page<EntityOption>> {
  const result = await companyService.listPage({
    page: params.page,
    page_size: params.page_size,
    filter: {
      search: params.search,
      sector_id: null,
      company_type_id: null,
      company_size: null,
      relation_state: null,
    },
  });
  return {
    ...result,
    items: result.items.map((company) => ({
      id: company.id,
      label: company.name,
      meta: company.city ?? undefined,
    })),
  };
}
