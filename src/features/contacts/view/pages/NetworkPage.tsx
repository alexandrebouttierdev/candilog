import { useState } from "react";
import { useContactsViewModel } from "../../viewmodel/useContactsViewModel";
import type { Contact } from "../../services/contactService";
import { ContactFormModal } from "../components/ContactFormModal";
import { ContactDetail } from "../components/ContactDetail";
import { roleMeta } from "../../model/roles";
import { ContextBarAccessory, ContextSearch } from "@/app/layout/ContextBar";
import {
  Button,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  MasterList,
  MasterListItem,
  MasterListTag,
  PageHeader,
  Pager,
  Skeleton,
  initials,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";

/** Écran Relations → Réseau : liste maître paginée et fiche détaillée. */
export function NetworkPage() {
  const vm = useContactsViewModel();
  const [form, setForm] = useState<{ ouvert: boolean; cible: Contact | null }>({
    ouvert: false,
    cible: null,
  });
  const [aDelete, setADelete] = useState<Contact | null>(null);

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextSearch
          value={vm.search}
          placeholder="Rechercher un contact…"
          onChange={vm.rechercher}
        />
      </ContextBarAccessory>

      <PageHeader
        icon="hub"
        title="Réseau"
        subtitle="Vos contacts professionnels"
        primary={
          <Button
            variant="primary"
            icon="person_add"
            onClick={() => setForm({ ouvert: true, cible: null })}
          >
            Nouveau contact
          </Button>
        }
      />

      <div className="flex min-h-0 flex-1">
        <MasterList
          title="Votre réseau"
          count={`${vm.total} ${vm.total > 1 ? "contacts" : "contact"}`}
          footer={
            vm.total > 0 ? (
              <Pager
                dense
                page={vm.page}
                page_size={vm.page_size}
                total={vm.total}
                label="contacts"
                onPageChange={vm.setPage}
              />
            ) : null
          }
        >
          {vm.isLoading ? (
            <ListSquelette />
          ) : vm.error ? (
            <div className="p-3">
              <ErrorBanner
                message={
                  vm.error instanceof AppError
                    ? vm.error.message
                    : "Le réseau n'a pas pu être chargé."
                }
                onRetry={vm.recharger}
              />
            </div>
          ) : vm.items.length === 0 ? (
            <EmptyState
              icon="hub"
              title={vm.search ? "Aucun résultat" : "Aucun contact"}
              description={
                vm.search
                  ? "Aucun contact ne correspond à cette recherche."
                  : "Ajoutez un interlocuteur pour garder trace de vos échanges."
              }
            />
          ) : (
            vm.items.map((contact) => (
              <MasterListItem
                key={contact.id}
                round
                initials={initials(contact.first_name, contact.name)}
                title={`${contact.first_name} ${contact.name}`}
                subtitle={
                  [contact.job_title, contact.company_name].filter(Boolean).join(" · ") ||
                  undefined
                }
                meta={
                  contact.tracking_role ? (
                    <MasterListTag {...roleMeta(contact.tracking_role)}>
                      {contact.tracking_role}
                    </MasterListTag>
                  ) : undefined
                }
                selected={contact.id === vm.selected_id}
                onSelect={() => vm.selectionner(contact.id)}
              />
            ))
          )}
        </MasterList>

        <div className="min-w-0 flex-1">
          {vm.selection ? (
            <ContactDetail
              contact={vm.selection}
              onEdit={() => setForm({ ouvert: true, cible: vm.selection })}
              onDelete={() => setADelete(vm.selection)}
            />
          ) : (
            <div className="flex h-full items-center justify-center">
              <EmptyState
                icon="ads_click"
                title="Aucun contact sélectionné"
                description="Choisissez un interlocuteur dans la liste pour afficher sa fiche."
              />
            </div>
          )}
        </div>
      </div>

      <ContactFormModal
        open={form.ouvert}
        contact={form.cible}
        busy={vm.isSaving}
        onClose={() => setForm({ ouvert: false, cible: null })}
        onSubmit={(values) =>
          form.cible
            ? vm.update({ id: form.cible.id, input: values })
            : vm.create(values)
        }
      />

      <ConfirmDialog
        open={aDelete !== null}
        title="Supprimer ce contact ?"
        description={`« ${aDelete ? `${aDelete.first_name} ${aDelete.name}` : ""} » sera définitivement retiré de votre réseau. Cette action est irréversible.`}
        note="La suppression est refusée si des candidatures ou des entretiens le référencent."
        busy={vm.isDeleting}
        onCancel={() => setADelete(null)}
        onConfirm={() => {
          const cible = aDelete;
          setADelete(null);
          if (cible) void vm.delete(cible.id);
        }}
      />
    </div>
  );
}

/** Squelette de la liste maître, aux dimensions des éléments réels. */
function ListSquelette() {
  return (
    <div role="status" aria-label="Chargement du réseau">
      {Array.from({ length: 6 }, (_, index) => (
        <div key={index} className="mb-1 flex items-center gap-[11px] px-3 py-[11px]">
          <Skeleton className="size-8 flex-none rounded-full" />
          <div className="flex-1">
            <Skeleton className="h-3 w-2/3" />
            <Skeleton className="mt-1.5 h-2.5 w-1/3" />
          </div>
        </div>
      ))}
    </div>
  );
}
