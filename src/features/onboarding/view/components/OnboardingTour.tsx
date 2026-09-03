import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { Button, Icon } from "@/shared/ui";
import { ONBOARDING_STEPS } from "../../model/steps";
import { OnboardingPreview } from "./OnboardingPreview";

/**
 * Tour d'accueil, affiché une seule fois : une étape par section de l'application.
 *
 * Volontairement **non-fermable** avant la dernière étape — ni Escape, ni clic extérieur,
 * ni croix — c'est la demande explicite du produit. `ModalHost` impose ces deux raccourcis
 * à toutes les modales (`useDismissable`) : ce composant est donc autonome plutôt qu'un
 * `ModalHost` détourné.
 */
export function OnboardingTour({ onFinish }: { onFinish: () => void }) {
  const [index, setIndex] = useState(0);
  const step = ONBOARDING_STEPS[index]!;
  const dernier = index === ONBOARDING_STEPS.length - 1;

  // Le focus doit rester dans le tour : sans lui, la tabulation continue de parcourir
  // l'arrière-plan atténué à l'ouverture, comme pour `ModalHost`. `Button` ne transmet pas
  // de `ref` (pas de `forwardRef`) : un identifiant stable évite d'y toucher pour ce seul
  // besoin.
  useEffect(() => {
    document.getElementById("onboarding-primary")?.focus();
  }, [index]);

  return createPortal(
    <div
      role="dialog"
      aria-modal="true"
      aria-label={step.title}
      className="fixed inset-0 z-[200] flex items-center justify-center bg-scrim/80 p-6 backdrop-blur-[3px]"
    >
      <div className="flex w-full max-w-[560px] flex-col overflow-hidden rounded-overlay border border-overlay bg-surface shadow-overlay">
        <div className="p-7 pb-5">
          {step.icon ? (
            <span className="mb-4 inline-flex size-10 items-center justify-center rounded-tile bg-accent-tint text-accent">
              <Icon name={step.icon} size={20} />
            </span>
          ) : null}
          <p className="text-eyebrow uppercase tracking-[0.07em] text-accent">{step.eyebrow}</p>
          <h2 className="mt-1.5 text-title text-ink">{step.title}</h2>
          <p className="mt-2.5 text-body leading-relaxed text-ink-muted">{step.description}</p>
          <div className="mt-5">
            <OnboardingPreview kind={step.kind} />
          </div>
        </div>

        <div className="flex items-center gap-4 border-t border-line bg-surface-alt px-7 py-4">
          <div className="flex flex-1 items-center gap-1.5" aria-hidden="true">
            {ONBOARDING_STEPS.map((entry, dotIndex) => (
              <span
                key={entry.kind}
                className={
                  dotIndex === index
                    ? "h-1.5 w-5 rounded-full bg-accent"
                    : "h-1.5 w-1.5 rounded-full bg-neutral-tint"
                }
              />
            ))}
          </div>
          <span className="sr-only" role="status">
            Étape {index + 1} sur {ONBOARDING_STEPS.length}
          </span>
          {index > 0 ? (
            <Button variant="secondary" size="dialog" onClick={() => setIndex((current) => current - 1)}>
              Précédent
            </Button>
          ) : null}
          <Button
            id="onboarding-primary"
            variant="primary"
            size="dialog"
            icon={dernier ? "rocket_launch" : "chevron_right"}
            onClick={() => {
              if (dernier) {
                onFinish();
                return;
              }
              setIndex((current) => current + 1);
            }}
          >
            {dernier ? "Commencer" : "Suivant"}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}
