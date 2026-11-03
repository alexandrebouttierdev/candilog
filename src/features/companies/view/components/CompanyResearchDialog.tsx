import { useState, type ReactNode } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { AiProgressPanel } from "@/features/ai";
import { CompanySizes, useReferentials } from "@/features/referentials";
import { openExternal } from "@/shared/services/external-link";
import { cn } from "@/shared/lib/cn";
import type {
  CompanyCandidate,
  CompanyResearchResult,
  FieldOrigin,
  ResearchedField,
} from "@/shared/types/generated/ai";
import {
  Banner,
  BetaBadge,
  Button,
  EmptyState,
  ErrorBanner,
  FormField,
  ModalHost,
  Select,
  Tag,
  TextInput,
} from "@/shared/ui";
import type { CompanyFormInput } from "../../model/schemas/company-form.schema";
import {
  companyReviewSchema,
  initialReview,
  replacesInput,
  reviewToFormPatch,
  type CompanyReviewInput,
  type CompanyReviewValues,
  type ResearchedFieldName,
} from "../../model/schemas/company-research.schema";
import type { CompanyResearchViewModel } from "../../viewmodel/useCompanyResearch";

const ORIGINS: Record<FieldOrigin, string> = {
  registry: "Annuaire des entreprises",
  wikidata: "Wikidata",
  web_search: "Recherche Web",
  ai: "Proposé par l'IA",
};

const LABELS: Record<ResearchedFieldName, string> = {
  name: "Nom",
  sector_id: "Secteur d'activité",
  company_type_id: "Type d'entreprise",
  company_size: "Taille",
  website: "Site web",
  city: "Ville",
  address: "Adresse",
};

const ORDER: readonly ResearchedFieldName[] = [
  "name",
  "sector_id",
  "company_type_id",
  "company_size",
  "website",
  "city",
  "address",
];

/** Ce qui quitte l'ordinateur pendant la recherche, dit avant qu'elle ne commence. */
const PRIVACY =
  "Le nom (et la ville, si vous la précisez) est envoyé à l'annuaire public des entreprises (data.gouv.fr), à Wikidata et à un moteur SearXNG ; le classement passe par votre fournisseur IA.";

/**
 * « Compléter avec l'IA » : progression, choix de l'établissement, vérification.
 *
 * Modale imbriquée dans le formulaire entreprise. Rien n'y est écrit avant « Remplir le
 * formulaire » ; les valeurs déjà saisies ne sont remplacées que si l'utilisateur coche
 * le champ.
 */
export function CompanyResearchDialog({
  vm,
  current,
  onApply,
}: {
  vm: CompanyResearchViewModel;
  /** Valeurs actuelles du formulaire entreprise, à préserver par défaut. */
  current: CompanyFormInput;
  onApply: (patch: Partial<CompanyFormInput>) => void;
}) {
  if (vm.phase === "review" && vm.result) {
    return <ReviewDialog vm={vm} result={vm.result} current={current} onApply={onApply} />;
  }

  const running = vm.phase === "searching" || vm.phase === "researching";
  const common = {
    open: vm.open,
    title: "Compléter avec l'IA",
    subtitle: vm.query.name,
    width: "640px",
    onClose: vm.close,
  };

  if (running) {
    return (
      <ModalHost {...common} cancelLabel={vm.stopping ? "Arrêt…" : "Arrêter"}>
        <div className="flex flex-col gap-3 pt-3">
          <Header />
          <AiProgressPanel progress={vm.progress} elapsedMs={vm.elapsedMs} />
          <p className="text-sub leading-normal text-tx-5">
            {vm.phase === "searching"
              ? "Recherche des établissements dans l'annuaire officiel."
              : "Recherche du site officiel et classement de l'entreprise. Selon le modèle, cela peut prendre une minute."}
          </p>
        </div>
      </ModalHost>
    );
  }

  if (vm.phase === "error" && vm.failure) {
    return (
      <ModalHost {...common} submitLabel="Réessayer" onSubmit={vm.retry} cancelLabel="Fermer">
        <div className="flex flex-col gap-3 pt-3">
          <ErrorBanner
            title={
              vm.failure.kind === "network"
                ? "Connexion impossible"
                : vm.failure.kind === "provider"
                  ? "Service indisponible"
                  : "Réponse inattendue"
            }
            message={vm.failure.message}
          />
          {vm.failure.kind === "network" ? (
            <p className="text-sub text-tx-5">
              Vérifiez votre connexion à Internet : l'annuaire des entreprises n'a pas pu être joint.
            </p>
          ) : null}
        </div>
      </ModalHost>
    );
  }

  if (vm.phase === "none") {
    return (
      <ModalHost
        {...common}
        submitLabel="Rechercher sur le Web seulement"
        onSubmit={vm.webOnly}
        cancelLabel="Fermer"
      >
        <div className="flex flex-col gap-4 pt-3">
          <EmptyState
            compact
            title="Aucun établissement trouvé"
            description="L'annuaire officiel ne connaît aucune entreprise ouverte sous ce nom. Précisez la ville, corrigez le nom, ou cherchez sur le Web : sans l'annuaire, la ville et l'adresse resteront vides plutôt que devinées."
          />
          <Refine vm={vm} />
        </div>
      </ModalHost>
    );
  }

  return (
    <ModalHost
      {...common}
      submitLabel="Continuer avec cet établissement"
      submitDisabled={vm.selected === null}
      onSubmit={vm.confirmChoice}
    >
      <div className="flex flex-col gap-4 pt-3">
        <Header />
        <p className="text-sub leading-normal text-tx-4">
          {vm.candidates.length} établissement{vm.candidates.length > 1 ? "s" : ""} trouvé
          {vm.candidates.length > 1 ? "s" : ""}
          {vm.truncated ? " (les plus proches seulement)" : ""} — choisissez celui que vous recherchez.
          Le siège social et les établissements locaux sont distingués.
        </p>
        <CandidateList vm={vm} />
        <Refine vm={vm} />
        <Button variant="link" className="self-start" onClick={vm.webOnly}>
          Aucun ne correspond : rechercher sur le Web seulement
        </Button>
      </div>
    </ModalHost>
  );
}

function Header() {
  return (
    <div className="flex items-start gap-2">
      <BetaBadge />
      <p className="text-sub leading-normal text-tx-5">{PRIVACY}</p>
    </div>
  );
}

function CandidateList({ vm }: { vm: CompanyResearchViewModel }) {
  return (
    <fieldset className="flex max-h-[320px] flex-col gap-1.5 overflow-y-auto">
      <legend className="sr-only">Établissements trouvés</legend>
      {vm.candidates.map((candidate) => (
        <CandidateRow
          key={candidate.siret}
          candidate={candidate}
          checked={vm.selected?.siret === candidate.siret}
          onSelect={() => vm.choose(candidate)}
        />
      ))}
    </fieldset>
  );
}

function CandidateRow({
  candidate,
  checked,
  onSelect,
}: {
  candidate: CompanyCandidate;
  checked: boolean;
  onSelect: () => void;
}) {
  return (
    <label
      className={cn(
        "flex cursor-pointer items-start gap-3 rounded-r8 border px-3 py-2.5",
        checked ? "border-ac bg-tint-ac-bg" : "border-bd-soft bg-panel hover:bg-hover",
      )}
    >
      <input
        type="radio"
        name="etablissement"
        className="mt-1 accent-ac"
        checked={checked}
        onChange={onSelect}
      />
      <span className="min-w-0 flex-1">
        <span className="flex flex-wrap items-center gap-1.5">
          <span className="text-row font-medium text-tx">{candidate.name}</span>
          {candidate.trade_name ? <span className="text-sub text-tx-4">· {candidate.trade_name}</span> : null}
          <Tag>{candidate.head_office ? "Siège social" : "Établissement"}</Tag>
        </span>
        <span className="mt-0.5 block text-sub text-tx-3">
          {[candidate.city, candidate.postal_code].filter(Boolean).join(" · ") || "Ville non diffusée"}
        </span>
        <span className="block text-sub text-tx-5">{candidate.address ?? "Adresse non diffusée par l'annuaire"}</span>
        {candidate.activity_label ? (
          <span className="block text-tiny text-tx-5">
            {candidate.activity_label}
            {candidate.activity_code ? ` · ${candidate.activity_code}` : ""}
          </span>
        ) : null}
      </span>
      <Button
        variant="link"
        className="text-tiny"
        onClick={(event) => {
          event.preventDefault();
          void openExternal(candidate.source_url);
        }}
      >
        Fiche
      </Button>
    </label>
  );
}

/** Relancer la recherche avec un autre nom ou une ville. */
function Refine({ vm }: { vm: CompanyResearchViewModel }) {
  const [name, setName] = useState(vm.query.name);
  const [location, setLocation] = useState(vm.query.location);
  return (
    <form
      className="grid grid-cols-[1fr_180px_auto] items-end gap-2 rounded-r8 bg-group p-3"
      onSubmit={(event) => {
        event.preventDefault();
        vm.search(name, location);
      }}
    >
      <FormField label="Nom recherché">
        {(props) => <TextInput {...props} value={name} onChange={(event) => setName(event.target.value)} />}
      </FormField>
      <FormField label="Ville ou code postal">
        {(props) => (
          <TextInput
            {...props}
            value={location}
            placeholder="Rennes, 35000…"
            onChange={(event) => setLocation(event.target.value)}
          />
        )}
      </FormField>
      <Button type="submit" variant="secondary" disabled={name.trim() === ""}>
        Relancer
      </Button>
    </form>
  );
}

function ReviewDialog({
  vm,
  result,
  current,
  onApply,
}: {
  vm: CompanyResearchViewModel;
  result: CompanyResearchResult;
  current: CompanyFormInput;
  onApply: (patch: Partial<CompanyFormInput>) => void;
}) {
  const referentials = useReferentials();
  const form = useForm<CompanyReviewInput, unknown, CompanyReviewValues>({
    resolver: zodResolver(companyReviewSchema),
    defaultValues: initialReview(result, current),
  });
  const found = ORDER.filter((field) => result.fields[field].value !== null).length;
  const submit = form.handleSubmit((values) => {
    onApply(reviewToFormPatch(values));
    vm.close();
  });

  return (
    <ModalHost
      open={vm.open}
      title="Vérifier les informations trouvées"
      subtitle={vm.selected ? `${vm.selected.name}${vm.selected.city ? ` · ${vm.selected.city}` : ""}` : vm.query.name}
      submitLabel="Remplir le formulaire"
      cancelLabel="Annuler"
      footer_note="Seuls les champs cochés sont remplis."
      width="680px"
      onClose={vm.close}
      onSubmit={() => void submit()}
    >
      <form onSubmit={(event) => void submit(event)} className="flex flex-col gap-4 pt-3">
        {result.warnings.length > 0 ? (
          <Banner
            tone="warning"
            title={found > 0 ? "Recherche partiellement réussie" : "Aucune information fiable trouvée"}
            message={result.warnings.join(" ")}
          >
            {result.warnings.some((warning) => warning.startsWith("IA indisponible")) ? (
              <Button variant="link" className="mt-1" onClick={vm.configureAi}>
                Configurer l'IA
              </Button>
            ) : null}
          </Banner>
        ) : null}

        <div className="flex flex-col divide-y divide-bd-soft rounded-r8 border border-bd-soft">
          {ORDER.map((field) => (
            <ReviewRow
              key={field}
              field={field}
              researched={result.fields[field]}
              replaces={replacesInput(field, current)}
              form={form}
              control={(props) =>
                field === "sector_id" ? (
                  <Select {...props} {...form.register("sector_id.value")}>
                    <option value="">Sélectionner…</option>
                    {referentials.data.sectors.map((sector) => (
                      <option key={sector.id} value={sector.id}>
                        {sector.name}
                      </option>
                    ))}
                  </Select>
                ) : field === "company_type_id" ? (
                  <Select {...props} {...form.register("company_type_id.value")}>
                    <option value="">Sélectionner…</option>
                    {referentials.data.company_types.map((type) => (
                      <option key={type.code} value={type.code}>
                        {type.name}
                      </option>
                    ))}
                  </Select>
                ) : field === "company_size" ? (
                  <Select {...props} {...form.register("company_size.value")}>
                    {CompanySizes.map((size) => (
                      <option key={size.value} value={size.value}>
                        {size.label}
                      </option>
                    ))}
                  </Select>
                ) : (
                  <TextInput {...props} {...form.register(`${field}.value`)} />
                )
              }
            />
          ))}
        </div>

        {result.sources.length > 0 ? (
          <section aria-label="Sources consultées">
            <p className="caps mb-1.5">Sources consultées</p>
            <ul className="flex flex-col gap-1">
              {result.sources.map((source) => (
                <li key={source.url} className="flex min-w-0 items-baseline gap-2 text-sub">
                  <Button variant="link" className="text-sub" onClick={() => void openExternal(source.url)}>
                    {source.label}
                  </Button>
                  <span className="truncate font-mono text-tiny text-tx-6">{source.url}</span>
                </li>
              ))}
            </ul>
          </section>
        ) : null}
      </form>
    </ModalHost>
  );
}

function ReviewRow({
  field,
  researched,
  replaces,
  form,
  control,
}: {
  field: ResearchedFieldName;
  researched: ResearchedField;
  replaces: boolean;
  form: ReturnType<typeof useForm<CompanyReviewInput, unknown, CompanyReviewValues>>;
  control: (props: { id: string; "aria-describedby": string | undefined; "aria-invalid": boolean }) => ReactNode;
}) {
  const missing = researched.value === null;
  const error = form.formState.errors[field]?.value?.message;
  return (
    <div className="grid grid-cols-[150px_1fr] gap-3 px-3 py-2.5">
      <div className="pt-[22px]">
        <label className={cn("flex items-center gap-2 text-sub", missing ? "text-tx-6" : "text-tx-3")}>
          <input type="checkbox" className="accent-ac" disabled={missing} {...form.register(`${field}.apply`)} />
          {replaces ? "Remplacer" : "Remplir"}
        </label>
      </div>
      <div className="min-w-0">
        {missing ? (
          <div>
            <p className="mb-[5px] text-sub font-mid text-tx-4">{LABELS[field]}</p>
            <p className="text-sub text-tx-5">Aucune information fiable trouvée — le champ reste tel quel.</p>
          </div>
        ) : (
          <FormField label={LABELS[field]} error={error}>
            {control}
          </FormField>
        )}
        {!missing && researched.origin ? (
          <p className="mt-1 flex flex-wrap items-baseline gap-x-1.5 text-tiny text-tx-5">
            <span className="font-medium text-tx-4">{ORIGINS[researched.origin]}</span>
            {researched.note ? <span>· {researched.note}</span> : null}
            {researched.source_url ? (
              <Button
                variant="link"
                className="text-tiny"
                onClick={() => {
                  if (researched.source_url) void openExternal(researched.source_url);
                }}
              >
                Voir la source
              </Button>
            ) : null}
          </p>
        ) : null}
      </div>
    </div>
  );
}
