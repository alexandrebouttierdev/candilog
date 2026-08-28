"use client";

import { Download } from "lucide-react";
import { useSyncExternalStore } from "react";

const RELEASE_BASE =
  "https://github.com/alexandrebouttierdev/candilog-releases/releases/latest/download";

type Platform = "windows" | "macos" | "linux" | "unknown";

const platformDownloads = {
  windows: {
    label: "Installer Candilog",
    href: `${RELEASE_BASE}/candilog-windows-latest.exe`,
  },
  macos: {
    label: "Installer Candilog",
    href: `${RELEASE_BASE}/candilog-macos-latest.dmg`,
  },
  linux: {
    label: "Installer Candilog",
    href: `${RELEASE_BASE}/candilog-ubuntu-latest.AppImage`,
  },
  unknown: {
    label: "Installer Candilog",
    href: "#telecharger",
  },
} satisfies Record<Platform, { label: string; href: string }>;

function detectPlatform(): Platform {
  const descriptor = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();

  if (descriptor.includes("win")) return "windows";
  if (descriptor.includes("mac")) return "macos";
  if (descriptor.includes("linux") || descriptor.includes("x11")) return "linux";
  return "unknown";
}

export function DownloadButton() {
  const platform = useSyncExternalStore(
    () => () => undefined,
    detectPlatform,
    () => "unknown" as const,
  );

  const download = platformDownloads[platform];

  return (
    <a className="action action-primary" href={download.href}>
      <Download aria-hidden="true" size={18} strokeWidth={2.2} />
      {download.label}
    </a>
  );
}
