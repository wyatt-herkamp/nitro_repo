/** Shared formatting helpers, so dates and sizes read the same on every page. */

/** `2024-03-05` — short, unambiguous, and stable across locales in width. */
export function formatDate(value: unknown): string {
  if (!value) return "—";
  const date = new Date(value as string);
  if (Number.isNaN(date.getTime())) return "—";
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function formatDateTime(value: unknown): string {
  if (!value) return "—";
  const date = new Date(value as string);
  if (Number.isNaN(date.getTime())) return "—";
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** "3 days ago" — for things where the gap matters more than the date, like a token's last use. */
export function formatRelative(value: unknown): string {
  if (!value) return "Never";
  const date = new Date(value as string);
  if (Number.isNaN(date.getTime())) return "—";

  const seconds = Math.round((date.getTime() - Date.now()) / 1000);
  const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  const steps: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 31_536_000],
    ["month", 2_592_000],
    ["day", 86_400],
    ["hour", 3600],
    ["minute", 60],
  ];

  for (const [unit, size] of steps) {
    if (Math.abs(seconds) >= size) {
      return formatter.format(Math.round(seconds / size), unit);
    }
  }
  return formatter.format(seconds, "second");
}

const SIZE_UNITS = ["B", "KiB", "MiB", "GiB", "TiB"];

/**
 * Binary units, because that is what an artifact repository's clients report and a jar that Maven
 * calls 1.2 MiB should not appear here as 1.3 MB.
 */
export function formatFileSize(bytes: unknown): string {
  const value = Number(bytes);
  if (!Number.isFinite(value) || value < 0) return "—";
  if (value === 0) return "0 B";

  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < SIZE_UNITS.length - 1) {
    size /= 1024;
    unit += 1;
  }
  // Bytes are always whole; everything above gets one decimal until it reaches three digits.
  const decimals = unit === 0 ? 0 : size >= 100 ? 0 : 1;
  return `${size.toFixed(decimals)} ${SIZE_UNITS[unit]}`;
}
