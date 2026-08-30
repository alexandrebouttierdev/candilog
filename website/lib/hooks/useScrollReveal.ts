"use client";

import { useEffect, useRef } from "react";

/**
 * Révèle les enfants directs au scroll : montée de 16px + fondu,
 * décalage de 80ms entre enfants. Respecte prefers-reduced-motion.
 *
 * Usage :
 *   const ref = useScrollReveal<HTMLDivElement>();
 *   <div ref={ref} className="grid gap-10"> ... </div>
 *
 * Les classes utilisées doivent exister dans globals.css :
 *   @layer utilities {
 *     .reveal      { opacity: 0; transform: translateY(16px); }
 *     .reveal-in   { opacity: 1; transform: translateY(0);
 *                    transition: opacity 640ms var(--ease-reveal),
 *                                transform 720ms var(--ease-reveal); }
 *   }
 */
export function useScrollReveal<T extends HTMLElement>(stagger = 80) {
  const ref = useRef<T>(null);

  useEffect(() => {
    const host = ref.current;
    if (!host) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    // Garde avant de masquer quoi que ce soit : sans IntersectionObserver, le
    // constructeur plus bas lèverait après que `.reveal` a posé `opacity: 0`, et la
    // page resterait blanche. Le prototype portait la même garde.
    if (!("IntersectionObserver" in window)) return;

    const kids = Array.from(host.children) as HTMLElement[];
    kids.forEach((el) => el.classList.add("reveal"));

    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          io.unobserve(entry.target);
          const el = entry.target as HTMLElement;
          const i = kids.indexOf(el);
          window.setTimeout(() => el.classList.add("reveal-in"), Math.max(0, i) * stagger);
        });
      },
      { threshold: 0.08, rootMargin: "0px 0px -6% 0px" }
    );

    kids.forEach((el) => io.observe(el));
    return () => io.disconnect();
  }, [stagger]);

  return ref;
}
