import type { InputHTMLAttributes, SelectHTMLAttributes, TextareaHTMLAttributes } from "react";
import { controlClasses } from "./FormField";

/**
 * Contrôles de saisie stylés aux jetons.
 *
 * Ils restent des `input` / `select` / `textarea` natifs : les composants de saisie
 * réimplémentés perdent la navigation clavier, la saisie vocale et la restitution par les
 * lecteurs d'écran, que le guide exige toutes.
 */

export function TextInput({
  invalid,
  className,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return <input className={controlClasses(invalid, className)} {...props} />;
}

export function TextArea({
  invalid,
  className,
  rows = 3,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { invalid?: boolean }) {
  return <textarea rows={rows} className={controlClasses(invalid, `py-2 ${className ?? ""}`)} {...props} />;
}

export function Select({
  invalid,
  className,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { invalid?: boolean }) {
  return (
    <select className={controlClasses(invalid, className)} {...props}>
      {children}
    </select>
  );
}
