import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type {
  ConflictChoice,
  RemoteProbe,
  SetupRequest,
  SyncProviderKind,
} from "@/shared/types/generated/sync";
import { syncService } from "../services/syncService";
import {
  SYNC_AVAILABILITY_KEY,
  SYNC_CONFLICTS_KEY,
  useSyncStatus,
} from "./useSyncStatus";

function detail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Dialogue de confirmation ouvert dans la section Synchronisation. */
export type SyncDialog = "disconnect" | "restore" | null;

/**
 * Orchestration de la section « Synchronisation et sauvegarde » : choix du fournisseur,
 * mise en service, état, conflits, restauration et déconnexion.
 */
export function useSyncSettingsViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const status = useSyncStatus();
  const availability = useQuery({ queryKey: SYNC_AVAILABILITY_KEY, queryFn: syncService.availability });
  const attached = status.data !== undefined && status.data.provider !== "local";
  const conflicts = useQuery({
    queryKey: SYNC_CONFLICTS_KEY,
    queryFn: syncService.conflicts,
    enabled: attached,
  });
  const [probe, setProbe] = useState<RemoteProbe | null>(null);
  const [recoveryKey, setRecoveryKey] = useState<string | null>(null);
  const [dialog, setDialog] = useState<SyncDialog>(null);

  const refresh = async () => {
    await queryClient.invalidateQueries({ queryKey: ["sync"] });
  };

  const chooseFolder = useMutation({
    mutationFn: syncService.chooseFolder,
    onSuccess: (result) => {
      if (result) setProbe(result);
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Dossier inutilisable", detail: detail(error) }),
  });

  const connect = useMutation({
    mutationFn: (provider: SyncProviderKind) => syncService.connect(provider),
    onSuccess: (result) => setProbe(result),
    onError: (error: unknown) => {
      if (error instanceof AppError && error.code === "CANCELLED") return;
      notify({ tone: "error", title: "Connexion impossible", detail: detail(error) });
    },
  });

  const reconnect = useMutation({
    mutationFn: async (provider: SyncProviderKind) => {
      await syncService.connect(provider);
      await syncService.now();
    },
    onSuccess: async () => {
      await refresh();
      notify({ tone: "success", title: "Connexion rétablie" });
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Reconnexion impossible", detail: detail(error) }),
  });

  const setup = useMutation({
    mutationFn: (request: SetupRequest) => syncService.setup(request),
    onSuccess: async (outcome) => {
      setProbe(null);
      if (outcome.recovery_key) setRecoveryKey(outcome.recovery_key);
      await queryClient.invalidateQueries();
      notify({
        tone: "success",
        title: "Synchronisation activée",
        detail: outcome.backup_file
          ? `Vos données précédentes ont été sauvegardées dans ${outcome.backup_file}.`
          : undefined,
      });
    },
  });

  const unlock = useMutation({
    mutationFn: ({ passphrase, recovery }: { passphrase: string | null; recovery: string | null }) =>
      syncService.unlock(passphrase, recovery),
    onSuccess: async () => {
      await refresh();
      notify({ tone: "success", title: "Espace déverrouillé" });
    },
  });

  const setAuto = useMutation({
    mutationFn: (enabled: boolean) => syncService.setAuto(enabled),
    onSuccess: refresh,
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Réglage non enregistré", detail: detail(error) }),
  });

  const syncNow = useMutation({
    mutationFn: syncService.now,
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Synchronisation impossible", detail: detail(error) }),
  });

  const disconnect = useMutation({
    mutationFn: syncService.disconnect,
    onSuccess: async () => {
      setDialog(null);
      await refresh();
      notify({ tone: "success", title: "Synchronisation désactivée", detail: "Vos données restent sur cet ordinateur." });
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Déconnexion impossible", detail: detail(error) }),
  });

  const restore = useMutation({
    mutationFn: syncService.restoreFromSpace,
    onSuccess: async (backupFile) => {
      setDialog(null);
      await queryClient.invalidateQueries();
      notify({
        tone: "success",
        title: "Données restaurées depuis l'espace",
        detail: `La version précédente est sauvegardée dans ${backupFile}.`,
      });
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Restauration impossible", detail: detail(error) }),
  });

  const resolve = useMutation({
    mutationFn: ({ id, choice }: { id: string; choice: ConflictChoice }) => syncService.resolve(id, choice),
    onSuccess: async () => {
      await queryClient.invalidateQueries();
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Conflit non résolu", detail: detail(error) }),
  });

  const resolveAll = useMutation({
    mutationFn: (choice: ConflictChoice) => syncService.resolveAll(choice),
    onSuccess: async (count) => {
      await queryClient.invalidateQueries();
      notify({ tone: "success", title: count === 1 ? "1 conflit résolu" : `${count} conflits résolus` });
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Conflits non résolus", detail: detail(error) }),
  });

  return {
    status: status.data,
    statusError: status.error instanceof AppError ? status.error.message : status.error ? "État illisible." : null,
    isLoading: status.isPending,
    availability: availability.data ?? { google_drive: false, one_drive: false },
    conflicts: conflicts.data ?? [],
    probe,
    recoveryKey,
    dialog,
    connecting: connect.isPending ? connect.variables : null,
    choosingFolder: chooseFolder.isPending,
    reconnecting: reconnect.isPending,
    settingUp: setup.isPending,
    unlocking: unlock.isPending,
    disconnecting: disconnect.isPending,
    restoring: restore.isPending,
    resolving: resolve.isPending || resolveAll.isPending,
    chooseFolder: () => chooseFolder.mutate(),
    connect: (provider: SyncProviderKind) => connect.mutate(provider),
    cancelConnect: () => void syncService.cancelConnect(),
    reconnect: (provider: SyncProviderKind) => reconnect.mutate(provider),
    closeProbe: () => setProbe(null),
    setup: setup.mutateAsync,
    closeRecoveryKey: () => setRecoveryKey(null),
    unlock: unlock.mutateAsync,
    setAuto: (enabled: boolean) => setAuto.mutate(enabled),
    syncNow: () => syncNow.mutate(),
    openDialog: setDialog,
    closeDialog: () => setDialog(null),
    disconnect: () => disconnect.mutate(),
    restore: () => restore.mutate(),
    resolve: (id: string, choice: ConflictChoice) => resolve.mutate({ id, choice }),
    resolveAll: (choice: ConflictChoice) => resolveAll.mutate(choice),
  };
}

export type SyncSettingsViewModel = ReturnType<typeof useSyncSettingsViewModel>;
