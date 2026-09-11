import { contactService } from "../services/contactService";
import type { Page } from "@/shared/types/page";
import type { EntityOption } from "@/shared/ui";

/** Page de contacts pour un EntityPicker — API publique de la feature. */
export async function fetchContactPickerPage(params: {
  page: number;
  page_size: number;
  search: string;
}): Promise<Page<EntityOption>> {
  const result = await contactService.listPage({
    ...params,
    tracking_role: null,
  });
  return {
    ...result,
    items: result.items.map((contact) => ({
      id: contact.id,
      label: `${contact.first_name} ${contact.name}`,
      meta: contact.company_name ?? undefined,
    })),
  };
}
