"use client";

import { useSyncExternalStore } from "react";

import { LineIcon } from "@/components/ui/LineIcon";

const KEY = "candilog-theme";

/* Le thème vit sur <html data-theme>, posé par le script anti-flash du layout avant
   le premier paint. C'est donc une source externe à React : on la lit avec
   useSyncExternalStore plutôt qu'en recopiant l'attribut dans un état au montage.
   Effet de bord utile — l'icône reste juste si le thème change ailleurs. */
function subscribe(onStoreChange: () => void) {
  const observer = new MutationObserver(onStoreChange);
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
  return () => observer.disconnect();
}

const isDark = () => document.documentElement.getAttribute("data-theme") === "dark";
// Rendu serveur : <html data-theme="light"> est la valeur du layout.
const isDarkOnServer = () => false;

export function ThemeToggle() {
  const dark = useSyncExternalStore(subscribe, isDark, isDarkOnServer);

  const toggle = () => {
    const next = dark ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", next);
    try {
      localStorage.setItem(KEY, next);
    } catch {
      // Stockage indisponible (navigation privée) : le thème vaut pour la session.
    }
  };

  const label = dark ? "Passer en thème clair" : "Passer en thème sombre";

  return (
    <button
      type="button"
      onClick={toggle}
      aria-label={label}
      title={label}
      className="grid size-[32px] shrink-0 place-items-center rounded-r7 border border-bd-menu bg-panel text-tx-3 transition-colors duration-[120ms] hover:border-bd-strong hover:bg-hover hover:text-tx"
    >
      <span
        className="block transition-transform duration-[320ms] ease-out-soft"
        style={{ transform: dark ? "rotate(180deg)" : "rotate(0deg)" }}
      >
        <LineIcon name={dark ? "sun" : "moon"} size={15} />
      </span>
    </button>
  );
}
