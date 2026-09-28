import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import { applyDisplayPrefs } from "./shared/lib/display-prefs";
import "./styles.css";

const container = document.getElementById("root");
if (!container) throw new Error("L'élément racine #root est absent de index.html.");

// Densité et animations avant le premier rendu : aucune ligne ne change de hauteur après
// l'affichage.
applyDisplayPrefs();

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
