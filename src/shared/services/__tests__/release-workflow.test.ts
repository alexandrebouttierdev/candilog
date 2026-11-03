import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const workflow = readFileSync(".github/workflows/release.yml", "utf8");
const quality = readFileSync(".github/workflows/quality.yml", "utf8");

describe("contrat du workflow de release", () => {
  it("bloque les builds tant que tous les contrôles qualité ne passent pas", () => {
    // Le contrôle vit dans `quality.yml`, rejoué par la release : une seule définition.
    expect(workflow).toMatch(/^ {2}quality:\n(?: {4}#.*\n)* {4}uses: \.\/\.github\/workflows\/quality\.yml$/m);
    for (const command of [
      "yarn install --immutable",
      "yarn lint",
      "yarn test",
      "yarn build",
      "cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check",
      "cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings",
      "cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets",
      "cargo deny --manifest-path src-tauri/Cargo.toml check",
    ]) {
      expect(quality).toContain(command);
    }
    expect(workflow).toMatch(/^ {2}build:\n {4}needs: \[prepare, quality\]$/m);
  });

  it("fige chaque action tierce sur un commit complet", () => {
    for (const source of [workflow, quality]) {
      expect(source).not.toMatch(/uses:\s*[^@\s]+@(v\d+|stable)\b/);
      for (const reference of source.matchAll(/uses:\s*[^@\s]+@([^\s#]+)/g)) {
        expect(reference[1]).toMatch(/^[0-9a-f]{40}$/);
      }
    }
  });

  it("accorde les écritures uniquement au job de publication", () => {
    expect(workflow).toMatch(/^permissions:\n {2}contents: read$/m);
    const publishStart = workflow.indexOf("  publish:");
    expect(publishStart).toBeGreaterThan(0);
    expect(workflow.slice(0, publishStart)).not.toMatch(/contents: write|id-token: write|attestations: write/);
    expect(workflow.slice(publishStart)).toMatch(/contents: write/);
    expect(workflow.slice(publishStart)).toMatch(/id-token: write/);
    expect(workflow.slice(publishStart)).toMatch(/attestations: write/);
  });

  it("contrôle aussi chaque push sur dev et chaque pull request, sans droit d'écriture", () => {
    expect(quality).toMatch(/^on:\n {2}push:\n {4}branches:\n {6}- dev\n {2}pull_request:\n {2}workflow_call:$/m);
    expect(quality).toMatch(/^permissions:\n {2}contents: read$/m);
    expect(quality).not.toMatch(/contents: write|id-token: write|attestations: write|secrets\./);
  });
});
