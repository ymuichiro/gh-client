import { useMemo } from "react";
import hljs from "highlight.js";

export type DiffViewMode = "inline" | "split";

interface DiffSyntaxPreviewProps {
  content: string;
  filename?: string | null;
  emptyLabel: string;
  viewMode: DiffViewMode;
}

interface HighlightedDiffLine {
  key: string;
  kind: "add" | "remove" | "context" | "meta";
  prefix: string;
  html: string;
}

const MAX_HIGHLIGHT_LINES = 2500;

const EXTENSION_LANGUAGE_MAP: Record<string, string> = {
  js: "javascript",
  jsx: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  ts: "typescript",
  tsx: "typescript",
  py: "python",
  rb: "ruby",
  php: "php",
  go: "go",
  rs: "rust",
  java: "java",
  kt: "kotlin",
  kts: "kotlin",
  swift: "swift",
  cs: "csharp",
  scala: "scala",
  c: "c",
  h: "c",
  cpp: "cpp",
  cxx: "cpp",
  cc: "cpp",
  hpp: "cpp",
  hxx: "cpp",
  sh: "bash",
  bash: "bash",
  zsh: "bash",
  fish: "bash",
  ps1: "powershell",
  sql: "sql",
  json: "json",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
  ini: "ini",
  xml: "xml",
  html: "xml",
  htm: "xml",
  css: "css",
  scss: "scss",
  less: "less",
  vue: "vue",
  svelte: "svelte",
  md: "markdown",
  markdown: "markdown",
  dockerfile: "dockerfile",
  makefile: "makefile",
  tf: "hcl",
};

export function DiffSyntaxPreview({
  content,
  filename,
  emptyLabel,
  viewMode,
}: DiffSyntaxPreviewProps): JSX.Element {
  const highlightedLines = useMemo(
    () => toHighlightedDiffLines(content, resolveLanguageFromFilename(filename)),
    [content, filename],
  );

  if (content.trim().length === 0) {
    return <pre className="diff-preview">{emptyLabel}</pre>;
  }

  if (viewMode === "split") {
    return renderSplitView(highlightedLines);
  }

  return renderInlineView(highlightedLines);
}

function toHighlightedDiffLines(content: string, language: string | null): HighlightedDiffLine[] {
  const rawLines = content.split("\n");
  const shouldHighlight = rawLines.length <= MAX_HIGHLIGHT_LINES;

  return rawLines.map((rawLine, index) => {
    const parsed = parseDiffLine(rawLine);
    const html =
      parsed.kind === "meta"
        ? escapeHtml(parsed.code)
        : shouldHighlight
          ? highlightCode(parsed.code, language)
          : escapeHtml(parsed.code);

    return {
      key: `${index}-${parsed.kind}-${parsed.prefix}`,
      kind: parsed.kind,
      prefix: parsed.prefix,
      html: html.length > 0 ? html : "&nbsp;",
    };
  });
}

function renderInlineView(lines: HighlightedDiffLine[]): JSX.Element {
  return (
    <div
      className="diff-preview diff-preview-code diff-preview-inline"
      role="region"
      aria-label="diff preview inline"
    >
      {lines.map((line) => (
        <div key={line.key} className={`diff-line ${line.kind}`}>
          <span className="diff-line-prefix">{line.prefix}</span>
          <span className="diff-line-content" dangerouslySetInnerHTML={{ __html: line.html }} />
        </div>
      ))}
    </div>
  );
}

function renderSplitView(lines: HighlightedDiffLine[]): JSX.Element {
  const rows = toSplitRows(lines);

  return (
    <div
      className="diff-preview diff-preview-code diff-preview-split"
      role="region"
      aria-label="diff preview split"
    >
      <div className="diff-split-header">
        <span>old</span>
        <span>new</span>
      </div>
      {rows.map((row) =>
        row.kind === "meta" ? (
          <div key={row.key} className="diff-split-meta">
            <span>{row.text}</span>
          </div>
        ) : (
          <div key={row.key} className="diff-split-row">
            {renderSplitCell(row.left, "left")}
            {renderSplitCell(row.right, "right")}
          </div>
        ),
      )}
    </div>
  );
}

function renderSplitCell(
  line: HighlightedDiffLine | null,
  side: "left" | "right",
): JSX.Element {
  if (!line) {
    return <div className={`diff-split-cell empty ${side}`} />;
  }

  return (
    <div className={`diff-split-cell ${line.kind} ${side}`}>
      <span className="diff-line-prefix">{line.prefix}</span>
      <span className="diff-line-content" dangerouslySetInnerHTML={{ __html: line.html }} />
    </div>
  );
}

function toSplitRows(
  lines: HighlightedDiffLine[],
): Array<
  | { key: string; kind: "meta"; text: string }
  | {
      key: string;
      kind: "code";
      left: HighlightedDiffLine | null;
      right: HighlightedDiffLine | null;
    }
> {
  const rows: Array<
    | { key: string; kind: "meta"; text: string }
    | {
        key: string;
        kind: "code";
        left: HighlightedDiffLine | null;
        right: HighlightedDiffLine | null;
      }
  > = [];

  let index = 0;
  while (index < lines.length) {
    const current = lines[index];
    if (current.kind === "meta") {
      rows.push({
        key: current.key,
        kind: "meta",
        text: decodeHtmlEntities(current.html),
      });
      index += 1;
      continue;
    }

    if (current.kind === "context") {
      rows.push({
        key: current.key,
        kind: "code",
        left: current,
        right: current,
      });
      index += 1;
      continue;
    }

    if (current.kind === "remove") {
      const next = lines[index + 1];
      if (next && next.kind === "add") {
        rows.push({
          key: `${current.key}-${next.key}`,
          kind: "code",
          left: current,
          right: next,
        });
        index += 2;
        continue;
      }

      rows.push({
        key: current.key,
        kind: "code",
        left: current,
        right: null,
      });
      index += 1;
      continue;
    }

    rows.push({
      key: current.key,
      kind: "code",
      left: null,
      right: current,
    });
    index += 1;
  }

  return rows;
}

function highlightCode(code: string, language: string | null): string {
  if (code.length === 0) {
    return "";
  }

  if (language && hljs.getLanguage(language)) {
    try {
      return hljs.highlight(code, { language, ignoreIllegals: true }).value;
    } catch (_error) {
      // Fallback to auto-detection below.
    }
  }

  try {
    return hljs.highlightAuto(code).value;
  } catch (_error) {
    return escapeHtml(code);
  }
}

function parseDiffLine(line: string): {
  kind: "add" | "remove" | "context" | "meta";
  prefix: string;
  code: string;
} {
  if (line.startsWith("diff ") || line.startsWith("@@") || line.startsWith("index ")) {
    return { kind: "meta", prefix: " ", code: line };
  }
  if (line.startsWith("+++ ") || line.startsWith("--- ")) {
    return { kind: "meta", prefix: " ", code: line };
  }
  if (line.startsWith("+")) {
    return { kind: "add", prefix: "+", code: line.slice(1) };
  }
  if (line.startsWith("-")) {
    return { kind: "remove", prefix: "-", code: line.slice(1) };
  }
  if (line.startsWith(" ")) {
    return { kind: "context", prefix: " ", code: line.slice(1) };
  }

  return { kind: "meta", prefix: " ", code: line };
}

function resolveLanguageFromFilename(filename?: string | null): string | null {
  if (!filename) {
    return null;
  }

  const normalized = filename.trim().toLowerCase();
  if (!normalized) {
    return null;
  }

  const parts = normalized.split("/");
  const basename = parts[parts.length - 1];
  if (basename === "dockerfile") {
    return "dockerfile";
  }
  if (basename === "makefile") {
    return "makefile";
  }

  const dotIndex = basename.lastIndexOf(".");
  if (dotIndex < 0 || dotIndex === basename.length - 1) {
    return null;
  }

  const extension = basename.slice(dotIndex + 1);
  return EXTENSION_LANGUAGE_MAP[extension] ?? null;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function decodeHtmlEntities(value: string): string {
  return value
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", "\"")
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&");
}
