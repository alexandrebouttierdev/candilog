import type { ReactNode } from "react";
import type { CvGenere, ProgressionIa } from "@/features/ia/model/types";
import { cn } from "@/shared/lib/cn";
import { Icon } from "@/shared/ui";

export function DocumentPanel({ title, icon, action, children, className }: { title: string; icon: string; action?: ReactNode; children: ReactNode; className?: string }) {
  return <section className={cn("overflow-hidden rounded-card border border-line bg-surface shadow-e1", className)}><header className="flex min-h-12 items-center gap-2 border-b border-line px-4"><Icon name={icon} size={17} className="text-accent" /><h2 className="min-w-0 flex-1 truncate text-section text-ink">{title}</h2>{action}</header>{children}</section>;
}

export function A4Preview({ cv, title = "Aperçu du document", children }: { cv?: CvGenere | undefined; title?: string; children?: ReactNode }) {
  return <div className="min-h-0 overflow-auto bg-surface-alt p-5"><article aria-label={title} className="mx-auto min-h-[720px] w-full max-w-[560px] bg-white px-10 py-9 text-[#20242c] shadow-e2"><p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-[#5b6ee1]">Candilog · document ciblé</p>{children ?? (cv ? <><h2 className="mt-4 text-[24px] font-semibold">{cv.resume || "Profil professionnel"}</h2><Section title="Compétences"><p>{cv.competences.join(" · ")}</p></Section><Section title="Expériences">{cv.experiences.map((item, index) => <div key={`${item.intitule}-${index}`} className="mb-4"><p className="font-semibold">{item.intitule} · {item.entreprise}</p><p className="mt-1 leading-relaxed text-[#505866]">{item.description}</p></div>)}</Section><Section title="Formations">{cv.formations.map((item, index) => <p key={`${item.diplome}-${index}`}><b>{item.diplome}</b> · {item.etablissement}</p>)}</Section></> : <div className="flex min-h-[590px] items-center justify-center text-center text-[#7b8493]">Le document apparaîtra ici après la génération.</div>)}</article></div>;
}

function Section({ title, children }: { title: string; children: ReactNode }) { return <section className="mt-7 border-t border-[#d9dee8] pt-3"><h3 className="mb-3 text-[11px] font-bold uppercase tracking-[0.12em] text-[#5b6ee1]">{title}</h3><div className="text-[12px] leading-relaxed">{children}</div></section>; }

export function IaProgress({ progress }: { progress: ProgressionIa | null }) {
  const value = progress?.progression ?? 5;
  return <div role="status" className="rounded-card border border-accent-border bg-accent-tint p-4"><div className="flex items-center gap-2"><Icon name="progress_activity" size={17} className="animate-spin text-accent" /><p className="flex-1 text-label font-medium text-ink">{progress?.etape ?? "Préparation…"}</p><span className="tabular text-meta text-accent">{value}%</span></div><div className="mt-3 h-1.5 overflow-hidden rounded-full bg-surface"><div style={{ width: `${value}%` }} className="h-full rounded-full bg-accent transition-[width] duration-300" /></div></div>;
}

export function ScoreBadge({ value, label = "Score ATS" }: { value: number; label?: string }) { return <div className="flex items-center gap-3"><span className={cn("tabular flex size-12 items-center justify-center rounded-full border-4 text-label font-semibold", value >= 70 ? "border-success text-success" : value >= 45 ? "border-warning text-warning" : "border-danger text-danger")}>{value}</span><div><p className="text-label font-medium text-ink">{label}</p><p className="text-meta text-ink-faint">sur 100</p></div></div>; }
