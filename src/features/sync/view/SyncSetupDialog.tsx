import { Controller, useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useState } from "react";
import { FormField, ModalHost, SegmentedControl, TextInput } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { AppError } from "@/shared/types/app-error";
import type { RemoteProbe, SetupOutcome, SetupRequest } from "@/shared/types/generated/sync";
import { formatMoment, PROVIDER_LABELS } from "../model/syncStatus";
import {
  createSpaceSchema,
  joinSpaceSchema,
  MIN_PASSPHRASE_CHARS,
  type CreateSpaceValues,
  type JoinSpaceInput,
  type JoinSpaceValues,
} from "../model/schemas/sync-secret.schema";

type Submit = (request: SetupRequest) => Promise<SetupOutcome>;

function message(error: unknown): string {
  return error instanceof AppError ? error.message : "La mise en service a échoué.";
}

/**
 * Première synchronisation avec un emplacement. Trois scénarios, jamais choisis en
 * silence : créer l'espace depuis cet ordinateur, récupérer celui qui existe en remplaçant
 * les données locales (sauvegarde prise avant), ou fusionner les deux.
 */
export function SyncSetupDialog({
  probe,
  busy,
  onSubmit,
  onClose,
}: {
  probe: RemoteProbe;
  busy: boolean;
  onSubmit: Submit;
  onClose: () => void;
}) {
  return probe.space_exists ? (
    <JoinSpaceDialog probe={probe} busy={busy} onSubmit={onSubmit} onClose={onClose} />
  ) : (
    <CreateSpaceDialog probe={probe} busy={busy} onSubmit={onSubmit} onClose={onClose} />
  );
}

function Location({ probe }: { probe: RemoteProbe }) {
  return (
    <p className="mb-3 rounded-r8 bg-group px-3 py-2 text-sub text-tx-4">
      <span className="text-tx-2">{PROVIDER_LABELS[probe.provider]}</span>
      {probe.provider === "folder" ? <span className="block font-mono text-caps break-all text-tx-5">{probe.location}</span> : null}
      {probe.space_created_at ? (
        <span className="block">Espace créé le {formatMoment(probe.space_created_at)}</span>
      ) : null}
    </p>
  );
}

function CreateSpaceDialog({ probe, busy, onSubmit, onClose }: { probe: RemoteProbe; busy: boolean; onSubmit: Submit; onClose: () => void }) {
  const [error, setError] = useState<string | null>(null);
  const form = useForm<CreateSpaceValues>({
    resolver: zodResolver(createSpaceSchema),
    defaultValues: { passphrase: "", confirmation: "" },
  });
  const submit = form.handleSubmit(async (values) => {
    setError(null);
    try {
      await onSubmit({ choice: "create_space", passphrase: values.passphrase, recovery_key: null });
    } catch (caught) {
      setError(message(caught));
    }
  });

  return (
    <ModalHost
      open
      title="Créer l'espace de synchronisation"
      subtitle="Les données de cet ordinateur y sont envoyées, chiffrées."
      submitLabel="Créer et synchroniser"
      busy={busy}
      submitDisabled={busy}
      width="500px"
      onClose={onClose}
      onSubmit={() => void submit()}
    >
      <Location probe={probe} />
      <p className="mb-3 text-sub leading-[1.5] text-tx-4">
        Tout est chiffré sur cet ordinateur avant l'envoi : le fournisseur ne peut pas lire vos
        candidatures, CV ni lettres. La phrase secrète sera demandée sur chaque nouvel
        ordinateur. <strong className="font-medium text-tx-2">Candilog ne la connaît pas et ne peut pas la retrouver</strong> :
        une clé de récupération vous sera remise juste après.
      </p>
      <div className="flex flex-col gap-3">
        <Controller
          control={form.control}
          name="passphrase"
          render={({ field, fieldState }) => (
            <FormField label="Phrase secrète" hint={`${MIN_PASSPHRASE_CHARS} caractères au moins`} error={fieldState.error?.message}>
              {(props) => <TextInput {...props} {...field} type="password" autoComplete="new-password" autoFocus invalid={fieldState.invalid} />}
            </FormField>
          )}
        />
        <Controller
          control={form.control}
          name="confirmation"
          render={({ field, fieldState }) => (
            <FormField label="Confirmer la phrase secrète" error={fieldState.error?.message ?? error ?? undefined}>
              {(props) => <TextInput {...props} {...field} type="password" autoComplete="new-password" invalid={fieldState.invalid} />}
            </FormField>
          )}
        />
      </div>
    </ModalHost>
  );
}

const CHOICES = [
  {
    value: "join_replace_local",
    title: "Récupérer les données de l'espace",
    detail:
      "Les données de cet ordinateur sont remplacées par celles de l'espace. Une sauvegarde complète est prise avant, dans le dossier des exports.",
  },
  {
    value: "join_merge",
    title: "Fusionner avec les données de cet ordinateur",
    detail:
      "Les deux ensembles sont réunis. Une fiche modifiée des deux côtés vous sera présentée pour arbitrage ; rien n'est écrasé en silence.",
  },
] as const;

function JoinSpaceDialog({ probe, busy, onSubmit, onClose }: { probe: RemoteProbe; busy: boolean; onSubmit: Submit; onClose: () => void }) {
  const [error, setError] = useState<string | null>(null);
  const form = useForm<JoinSpaceInput, unknown, JoinSpaceValues>({
    resolver: zodResolver(joinSpaceSchema),
    // Sans données locales, les deux scénarios reviennent au même : rien à remplacer.
    defaultValues: {
      ...(probe.local_has_data ? {} : { choice: "join_merge" as const }),
      method: "passphrase",
      secret: "",
    },
  });
  const method = useWatch({ control: form.control, name: "method" });
  const submit = form.handleSubmit(async (values) => {
    setError(null);
    try {
      await onSubmit({
        choice: values.choice,
        passphrase: values.method === "passphrase" ? values.secret : null,
        recovery_key: values.method === "recovery" ? values.secret : null,
      });
    } catch (caught) {
      setError(message(caught));
    }
  });

  return (
    <ModalHost
      open
      title="Rejoindre l'espace existant"
      subtitle="Un espace Candilog existe déjà à cet emplacement."
      submitLabel="Rejoindre"
      busy={busy}
      submitDisabled={busy}
      width="520px"
      onClose={onClose}
      onSubmit={() => void submit()}
    >
      <Location probe={probe} />
      {probe.local_has_data ? (
        <Controller
          control={form.control}
          name="choice"
          render={({ field, fieldState }) => (
            <fieldset className="mb-3">
              <legend className="mb-[5px] text-sub font-mid text-tx-4">Cet ordinateur contient déjà des données</legend>
              <div className="flex flex-col gap-1.5">
                {CHOICES.map((choice) => (
                  <label
                    key={choice.value}
                    className={cn(
                      "flex cursor-pointer items-start gap-2.5 rounded-r8 border px-3 py-2.5",
                      field.value === choice.value ? "border-ac bg-ac-soft" : "border-bd-menu",
                    )}
                  >
                    <input
                      type="radio"
                      name={field.name}
                      value={choice.value}
                      checked={field.value === choice.value}
                      onChange={() => field.onChange(choice.value)}
                      className="mt-[3px] accent-[var(--ac)]"
                    />
                    <span>
                      <span className="block text-ui font-medium text-tx-2">{choice.title}</span>
                      <span className="block text-sub leading-[1.45] text-tx-5">{choice.detail}</span>
                    </span>
                  </label>
                ))}
              </div>
              {fieldState.error ? <p role="alert" className="mt-1 text-sub text-st-c">{fieldState.error.message}</p> : null}
            </fieldset>
          )}
        />
      ) : (
        <p className="mb-3 text-sub text-tx-4">Cet ordinateur n'a pas encore de données : il reçoit celles de l'espace.</p>
      )}
      <div className="mb-2 flex items-center justify-between gap-3">
        <span className="text-sub font-mid text-tx-4">Déverrouiller avec</span>
        <Controller
          control={form.control}
          name="method"
          render={({ field }) => (
            <SegmentedControl
              dense
              label="Secret de l'espace"
              value={field.value}
              onChange={field.onChange}
              options={[
                { value: "passphrase", label: "Phrase secrète" },
                { value: "recovery", label: "Clé de récupération" },
              ]}
            />
          )}
        />
      </div>
      <Controller
        control={form.control}
        name="secret"
        render={({ field, fieldState }) => (
          <FormField
            label={method === "passphrase" ? "Phrase secrète" : "Clé de récupération"}
            error={fieldState.error?.message ?? error ?? undefined}
          >
            {(props) => (
              <TextInput
                {...props}
                {...field}
                type={method === "passphrase" ? "password" : "text"}
                autoComplete="off"
                spellCheck={false}
                className={method === "recovery" ? "font-mono" : undefined}
                invalid={fieldState.invalid}
              />
            )}
          </FormField>
        )}
      />
    </ModalHost>
  );
}
