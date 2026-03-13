import { useEffect, useMemo, useState } from "react";

import { ConfirmModal } from "./ConfirmModal";
import { LoadingIndicator } from "./LoadingIndicator";
import { executeCommand, ExecutionError } from "../core/executor";
import { resolveEnvelopePermission } from "../core/permissions";
import { useI18n } from "../core/i18n";
import type { CommandId, } from "../core/commandIds";
import type {
  CommandField,
  CommandPermission,
  CommandSelectionOptions,
  CommandSpec,
  FieldOption,
  FrontendInvokeError,
} from "../core/types";

export interface CommandExecutionEvent {
  commandId: CommandId;
  requestId: string;
  payload: Record<string, unknown>;
  status: "success" | "error";
  data?: unknown;
  error?: FrontendInvokeError;
}

interface CommandFormProps {
  spec: CommandSpec;
  owner: string;
  repo: string;
  repoPermission: CommandPermission | null;
  selectionOptions: CommandSelectionOptions;
  onExecuted: (event: CommandExecutionEvent) => void;
  onInspect: (title: string, value: unknown) => void;
}

export function CommandForm({
  spec,
  owner,
  repo,
  repoPermission,
  selectionOptions,
  onExecuted,
  onInspect,
}: CommandFormProps): JSX.Element {
  const { t } = useI18n();
  const [values, setValues] = useState<Record<string, string | boolean>>({});
  const [rawPayload, setRawPayload] = useState("{}");
  const [running, setRunning] = useState(false);
  const [runningSeconds, setRunningSeconds] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [showConfirm, setShowConfirm] = useState(false);
  const [lastResponse, setLastResponse] = useState<unknown>(null);
  const usesOwnerField = useMemo(() => hasField(spec.fields, "owner"), [spec.fields]);
  const usesRepoField = useMemo(() => hasField(spec.fields, "repo"), [spec.fields]);

  const effectivePermission = useMemo(
    () => resolveEnvelopePermission(spec.requiredPermission, repoPermission, spec.needsRepoContext),
    [spec.requiredPermission, repoPermission, spec.needsRepoContext],
  );

  useEffect(() => {
    if (!usesOwnerField && !usesRepoField) {
      return;
    }

    setValues((prev) => {
      let changed = false;
      const next: Record<string, string | boolean> = { ...prev };

      if (usesOwnerField && prev.owner !== owner) {
        next.owner = owner;
        changed = true;
      }

      if (usesRepoField && prev.repo !== repo) {
        next.repo = repo;
        changed = true;
      }

      return changed ? next : prev;
    });
  }, [owner, repo, usesOwnerField, usesRepoField]);

  useEffect(() => {
    if (!running) {
      setRunningSeconds(0);
      return;
    }

    const startedAt = Date.now();
    const timer = window.setInterval(() => {
      setRunningSeconds(Math.floor((Date.now() - startedAt) / 1000));
    }, 1_000);

    return () => {
      window.clearInterval(timer);
    };
  }, [running]);

  const canExecute = Boolean(effectivePermission);

  const onChangeField = (field: CommandField, value: string | boolean) => {
    setValues((prev) => ({ ...prev, [field.name]: value }));
  };

  const execute = async () => {
    if (!canExecute || running) {
      return;
    }

    setRunning(true);
    setErrorMessage(null);

    try {
      const payload = buildPayload(spec.fields, values, rawPayload);
      const result = await executeCommand(spec.id, payload, { permission: effectivePermission ?? undefined });
      setLastResponse(result.data);
      onExecuted({
        commandId: spec.id,
        requestId: result.requestId,
        payload: result.payload,
        status: "success",
        data: result.data,
      });
    } catch (error) {
      if (error instanceof ExecutionError) {
        setErrorMessage(`[${error.detail.code}] ${error.detail.message}`);
        onExecuted({
          commandId: spec.id,
          requestId: error.detail.request_id,
          payload: {},
          status: "error",
          error: error.detail,
        });
      } else if (error instanceof Error) {
        setErrorMessage(error.message);
        onExecuted({
          commandId: spec.id,
          requestId: "",
          payload: {},
          status: "error",
          error: {
            code: "validation_error",
            message: error.message,
            retryable: false,
            fingerprint: "frontend",
            request_id: "",
            command_id: spec.id,
          },
        });
      } else {
        const fallback = String(error);
        setErrorMessage(fallback);
      }
    } finally {
      setRunning(false);
      setShowConfirm(false);
    }
  };

  const handleExecuteClick = () => {
    if (spec.destructive) {
      setShowConfirm(true);
      return;
    }

    void execute();
  };

  return (
    <section className="command-card" data-testid={`command-${spec.id}`}>
      <header className="command-header">
        <div>
          <h3>{spec.title}</h3>
          <p>{spec.description}</p>
        </div>
        <div className="tag-row">
          <span className="tag">{spec.requiredPermission}</span>
          <span className={spec.destructive ? "tag danger" : "tag"}>{spec.destructive ? "destructive" : "safe"}</span>
          <span className="tag">{spec.exposure}</span>
        </div>
      </header>

      {spec.fields.length === 0 ? (
        <label>
          <span>{t("common.raw_json")}</span>
          <textarea
            className="input textarea"
            data-field="raw_payload"
            value={rawPayload}
            onChange={(event) => setRawPayload(event.target.value)}
            rows={4}
          />
        </label>
      ) : (
        <div className="field-grid">
          {spec.fields.map((field) =>
            renderField(
              spec,
              field,
              values[field.name],
              onChangeField,
              selectionOptions,
              t,
            ),
          )}
        </div>
      )}

      {!canExecute ? <p className="warn-text">{t("status.permission_missing")}</p> : null}
      {running ? (
        <LoadingIndicator
          size="sm"
          label={`${t("common.loading")} ${t("status.cancel_unavailable")} (${runningSeconds}s)`}
        />
      ) : null}
      {errorMessage ? <p className="error-text">{errorMessage}</p> : null}

      <div className="row gap-sm">
        <button type="button" className="btn" disabled={!canExecute || running} onClick={handleExecuteClick}>
          {t("common.execute")}
        </button>
        {lastResponse !== null ? (
          <button
            type="button"
            className="btn secondary"
            onClick={() => onInspect(`${spec.id} ${t("common.response")}`, lastResponse)}
          >
            {t("common.response")}
          </button>
        ) : null}
      </div>

      <ConfirmModal
        open={showConfirm}
        commandId={spec.id}
        onCancel={() => setShowConfirm(false)}
        onConfirm={() => {
          void execute();
        }}
      />
    </section>
  );
}

function renderField(
  spec: CommandSpec,
  field: CommandField,
  value: string | boolean | undefined,
  onChangeField: (field: CommandField, value: string | boolean) => void,
  selectionOptions: CommandSelectionOptions,
  t: (key: string) => string,
): JSX.Element {
  const dynamicOptions = resolveDynamicOptions(spec, field, selectionOptions);

  if (field.type === "boolean") {
    return (
      <label key={field.name} className="checkbox-row">
        <input
          type="checkbox"
          data-field={field.name}
          checked={value === true}
          onChange={(event) => onChangeField(field, event.target.checked)}
        />
        <span>{field.label}</span>
      </label>
    );
  }

  if (field.type === "select" || dynamicOptions !== null) {
    const options = field.type === "select" ? (field.options ?? []) : dynamicOptions ?? [];

    return (
      <label key={field.name}>
        <span>
          {field.label} {field.required ? `(${t("common.required")})` : `(${t("common.optional")})`}
        </span>
        <select
          className="input"
          data-field={field.name}
          value={typeof value === "string" ? value : ""}
          onChange={(event) => onChangeField(field, event.target.value)}
        >
          <option value="">--</option>
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>
    );
  }

  if (field.type === "textarea" || field.type === "string_list" || field.type === "json") {
    return (
      <label key={field.name}>
        <span>
          {field.label} {field.required ? `(${t("common.required")})` : `(${t("common.optional")})`}
        </span>
        <textarea
          className="input textarea"
          data-field={field.name}
          value={typeof value === "string" ? value : ""}
          onChange={(event) => onChangeField(field, event.target.value)}
          placeholder={field.placeholder}
          rows={3}
        />
      </label>
    );
  }

  return (
    <label key={field.name}>
      <span>
        {field.label} {field.required ? `(${t("common.required")})` : `(${t("common.optional")})`}
      </span>
      <input
        className="input"
        data-field={field.name}
        value={typeof value === "string" ? value : ""}
        type={field.type === "number" ? "number" : "text"}
        min={field.min}
        placeholder={field.placeholder}
        onChange={(event) => onChangeField(field, event.target.value)}
      />
    </label>
  );
}

function buildPayload(
  fields: CommandField[],
  values: Record<string, string | boolean>,
  rawPayload: string,
): Record<string, unknown> {
  if (fields.length === 0) {
    const parsed = JSON.parse(rawPayload || "{}");
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      throw new Error("raw payload must be a JSON object");
    }
    return parsed as Record<string, unknown>;
  }

  const payload: Record<string, unknown> = {};

  for (const field of fields) {
    const current = values[field.name];

    if (field.type === "boolean") {
      if (typeof current === "boolean") {
        payload[field.name] = current;
      }
      continue;
    }

    const text = typeof current === "string" ? current.trim() : "";

    if (!text) {
      if (field.required) {
        throw new Error(`missing required field: ${field.name}`);
      }
      continue;
    }

    if (field.type === "number") {
      const parsedNumber = Number(text);
      if (Number.isNaN(parsedNumber)) {
        throw new Error(`invalid number field: ${field.name}`);
      }
      payload[field.name] = parsedNumber;
      continue;
    }

    if (field.type === "string_list") {
      payload[field.name] = text
        .split(/[\n,]/)
        .map((item) => item.trim())
        .filter((item) => item.length > 0);
      continue;
    }

    if (field.type === "json") {
      payload[field.name] = JSON.parse(text);
      continue;
    }

    payload[field.name] = text;
  }

  return payload;
}

function hasField(fields: CommandField[], name: string): boolean {
  return fields.some((field) => field.name === name);
}

function resolveDynamicOptions(
  spec: CommandSpec,
  field: CommandField,
  selectionOptions: CommandSelectionOptions,
): FieldOption[] | null {
  if (field.name === "owner") {
    return toStringOptions(selectionOptions.ownerOptions);
  }

  if (field.name === "repo") {
    return toStringOptions(selectionOptions.repoOptions);
  }

  if (
    field.name === "branch" ||
    field.name === "from_branch" ||
    field.name === "base" ||
    field.name === "default_branch" ||
    field.name === "target"
  ) {
    return toStringOptions(selectionOptions.branchOptions);
  }

  if (field.name === "tag") {
    return toStringOptions(selectionOptions.releaseTagOptions);
  }

  if (field.name === "run_id") {
    return toNumberOptions(selectionOptions.runIdOptions);
  }

  if (field.name === "number" && spec.id.startsWith("pr.")) {
    return toNumberOptions(selectionOptions.pullRequestNumberOptions);
  }

  if (field.name === "number" && spec.id.startsWith("issue.")) {
    return toNumberOptions(selectionOptions.issueNumberOptions);
  }

  return null;
}

function toStringOptions(values: string[]): FieldOption[] {
  return values.map((value) => ({ label: value, value }));
}

function toNumberOptions(values: number[]): FieldOption[] {
  return values.map((value) => ({ label: String(value), value: String(value) }));
}
