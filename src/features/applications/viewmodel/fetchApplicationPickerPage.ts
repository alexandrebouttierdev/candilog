import { applicationService } from "../services/applicationService";
import { EMPTY_FILTER } from "../model/schemas/application-filter.schema";
import type { Page } from "@/shared/types/page";
import type { EntityOption } from "@/shared/ui";

/** Page de candidatures pour un EntityPicker. */
export async function fetchApplicationPickerPage(params: {
  page: number;
  page_size: number;
  search: string;
}): Promise<Page<EntityOption>> {
  const result = await applicationService.listPage({
    page: params.page,
    page_size: params.page_size,
    filter: { ...EMPTY_FILTER, search: params.search, sort: "date", descending: true, ids: [] },
  });
  return {
    ...result,
    items: result.items.map((application) => ({
      id: application.id,
      label: application.job_title,
      meta: application.company_name ?? undefined,
    })),
  };
}

export function fetchApplicationPickerDetail(id: string) {
  return applicationService.get(id);
}
