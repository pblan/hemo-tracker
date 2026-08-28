export type ParsedInput =
  | { kind: "number"; value: number; normalized: string }
  | { kind: "date"; value: string; normalized: string }
  | { kind: "text"; value: string; normalized: string }
  | { kind: "ambiguous"; candidates: string[] };

export function parseMeasurementInput(
  input: string,
  locale: "de-DE" | "en-US",
): ParsedInput {
  const value = input.trim();
  if (!value) return { kind: "text", value: "", normalized: "" };
  const date = parseDate(value, locale);
  if (date) return date;
  const numeric = parseNumber(value, locale);
  if (numeric) return numeric;
  return { kind: "text", value, normalized: value };
}

function parseNumber(
  input: string,
  locale: "de-DE" | "en-US",
): ParsedInput | null {
  if (!/^[+-]?[\d.,\s]+$/.test(input)) return null;
  const compact = input.replace(/\s/g, "");
  const hasComma = compact.includes(",");
  const hasDot = compact.includes(".");
  if (
    hasComma &&
    hasDot &&
    compact.lastIndexOf(",") !== compact.lastIndexOf(".")
  ) {
    const decimal =
      compact.lastIndexOf(",") > compact.lastIndexOf(".") ? "," : ".";
    const thousands = decimal === "," ? "." : ",";
    const normalized = compact.replaceAll(thousands, "").replace(decimal, ".");
    return numberResult(normalized);
  }
  if (hasComma && hasDot)
    return {
      kind: "ambiguous",
      candidates: [compact.replaceAll(",", ""), compact.replaceAll(".", "")],
    };
  if (hasComma)
    return numberResult(
      locale === "de-DE"
        ? compact.replace(",", ".")
        : compact.replaceAll(",", ""),
    );
  if (hasDot)
    return numberResult(
      locale === "en-US" ? compact : compact.replaceAll(".", ""),
    );
  return numberResult(compact);
}

function numberResult(normalized: string): ParsedInput | null {
  const value = Number(normalized);
  return Number.isFinite(value)
    ? { kind: "number", value, normalized: String(value) }
    : null;
}

function parseDate(
  input: string,
  locale: "de-DE" | "en-US",
): ParsedInput | null {
  const match = input.match(/^(\d{1,2})[./-](\d{1,2})[./-](\d{4})$/);
  if (!match) return null;
  const [, first = "", second = "", year = ""] = match;
  if (first.length === 1 || second.length === 1) return null;
  if (locale === "en-US" && Number(first) <= 12 && Number(second) <= 12)
    return {
      kind: "ambiguous",
      candidates: [`${year}-${first}-${second}`, `${year}-${second}-${first}`],
    };
  const day = locale === "de-DE" ? first : second;
  const month = locale === "de-DE" ? second : first;
  return {
    kind: "date",
    value: input,
    normalized: `${year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`,
  };
}
