import { useState } from "react";
import { useContactsViewModel } from "../../viewmodel/useContactsViewModel";
import type { Contact } from "../../services/contact.service";
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
  initiales,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";

/** Écran Relations → Réseau : liste maître paginée et fiche détaillée. */
export function ReseauPage() {
  const vm = useContactsViewModel();
  const [formulaire, setFormulaire] = useState<{ ouvert: boolean; cible: Contact | null }>({
    ouvert: false,
    cible: null,
  });
  const [aSupprimer, setASupprimer] = useState<Contact | null>(null);

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
            onClick={() => setFormulaire({ ouvert: true, cible: null })}
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
                pageSize={vm.pageSize}
                total={vm.total}
                label="contacts"
                onPageChange={vm.setPage}
              />
            ) : null
          }
        >
          {vm.isLoading ? (
            <ListeSquelette />
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
                initials={initiales(contact.prenom, contact.nom)}
                title={`${contact.prenom} ${contact.nom}`}
                subtitle={
                  [contact.poste, contact.entrepriseNom].filter(Boolean).join(" · ") ||
                  undefined
                }
                meta={
                  contact.roleSuivi ? (
                    <MasterListTag {...roleMeta(contact.roleSuivi)}>
                      {contact.roleSuivi}
                    </MasterListTag>
                  ) : undefined
                }
                selected={contact.id === vm.selectedId}
                onSelect={() => vm.selectionner(contact.id)}
              />
            ))
          )}
        </MasterList>

        <div className="min-w-0 flex-1">
          {vm.selection ? (
            <ContactDetail
              contact={vm.selection}
              onEdit={() => setFormulaire({ ouvert: true, cible: vm.selection })}
              onDelete={() => setASupprimer(vm.selection)}
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
        open={formulaire.ouvert}
        contact={formulaire.cible}
        busy={vm.isSaving}
        onClose={() => setFormulaire({ ouvert: false, cible: null })}
        onSubmit={(valeurs) =>
          formulaire.cible
            ? vm.modifier({ id: formulaire.cible.id, input: valeurs })
            : vm.creer(valeurs)
        }
      />

      <ConfirmDialog
        open={aSupprimer !== null}
        title="Supprimer ce contact ?"
        description={`« ${aSupprimer ? `${aSupprimer.prenom} ${aSupprimer.nom}` : ""} » sera définitivement retiré de votre réseau. Cette action est irréversible.`}
        note="La suppression est refusée si des candidatures ou des entretiens le référencent."
        busy={vm.isDeleting}
        onCancel={() => setASupprimer(null)}
        onConfirm={() => {
          const cible = aSupprimer;
          setASupprimer(null);
          if (cible) void vm.supprimer(cible.id);
        }}
      />
    </div>
  );
}

/** Squelette de la liste maître, aux dimensions des éléments réels. */
function ListeSquelette() {
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
