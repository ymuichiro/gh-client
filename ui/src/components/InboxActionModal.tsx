import { useEffect, useMemo, useState } from "react";
import { LoadingIndicator } from "./LoadingIndicator";

interface ModalOption {
  label: string;
  value: string;
}

export interface InboxActionModalField {
  name: string;
  label: string;
  type: "text" | "textarea" | "select";
  required?: boolean;
  placeholder?: string;
  options?: ModalOption[];
  initialValue?: string;
  multiple?: boolean;
}

interface InboxActionModalProps {
  open: boolean;
  title: string;
  description?: string;
  fields?: InboxActionModalField[];
  previewItems?: string[];
  confirmLabel: string;
  cancelLabel: string;
  confirmToken?: string;
  tokenHint?: string;
  tokenLabel?: string;
  tokenPlaceholder?: string;
  requiredFieldMessage: (label: string) => string;
  tokenMismatchMessage: string;
  danger?: boolean;
  running?: boolean;
  runningLabel?: string;
  errorMessage?: string | null;
  onCancel: () => void;
  onConfirm: (values: Record<string, string>) => Promise<void> | void;
}

const EMPTY_FIELDS: InboxActionModalField[] = [];
const EMPTY_PREVIEW_ITEMS: string[] = [];

export function InboxActionModal({
  open,
  title,
  description,
  fields,
  previewItems,
  confirmLabel,
  cancelLabel,
  confirmToken,
  tokenHint,
  tokenLabel,
  tokenPlaceholder,
  requiredFieldMessage,
  tokenMismatchMessage,
  danger = false,
  running = false,
  runningLabel,
  errorMessage = null,
  onCancel,
  onConfirm,
}: InboxActionModalProps): JSX.Element | null {
  const resolvedFields = fields ?? EMPTY_FIELDS;
  const resolvedPreviewItems = previewItems ?? EMPTY_PREVIEW_ITEMS;
  const [values, setValues] = useState<Record<string, string>>({});
  const [tokenInput, setTokenInput] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    const nextValues: Record<string, string> = {};
    for (const field of resolvedFields) {
      nextValues[field.name] = field.initialValue ?? "";
    }
    setValues(nextValues);
    setTokenInput("");
    setValidationError(null);
  }, [open, resolvedFields]);

  const tokenMatched = useMemo(() => {
    if (!confirmToken) {
      return true;
    }

    return tokenInput.trim() === confirmToken;
  }, [confirmToken, tokenInput]);

  if (!open) {
    return null;
  }

  const submit = async () => {
    setValidationError(null);

    for (const field of resolvedFields) {
      const raw = values[field.name] ?? "";
      if (field.required && raw.trim().length === 0) {
        setValidationError(requiredFieldMessage(field.label));
        return;
      }
    }

    if (!tokenMatched) {
      setValidationError(tokenMismatchMessage);
      return;
    }

    await onConfirm(values);
  };

  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true">
      <div className="modal-panel inbox-modal-panel">
        <h3>{title}</h3>
        {description ? <p>{description}</p> : null}

        {resolvedPreviewItems.length > 0 ? (
          <div className="inbox-modal-preview">
            {resolvedPreviewItems.map((item) => (
              <code key={item} className="confirm-token">
                {item}
              </code>
            ))}
          </div>
        ) : null}

        {resolvedFields.map((field) => (
          <label key={field.name}>
            <span>{field.label}</span>
            {field.type === "textarea" ? (
              <textarea
                className="input textarea"
                value={values[field.name] ?? ""}
                onChange={(event) =>
                  setValues((current) => ({ ...current, [field.name]: event.target.value }))
                }
                placeholder={field.placeholder}
                rows={4}
              />
            ) : field.type === "select" ? (
              <select
                className="input"
                value={
                  field.multiple
                    ? parseCommaSeparated(values[field.name] ?? "")
                    : values[field.name] ?? ""
                }
                multiple={field.multiple}
                size={
                  field.multiple
                    ? Math.min(Math.max(field.options?.length ?? 3, 3), 8)
                    : undefined
                }
                onChange={(event) => {
                  if (field.multiple) {
                    const selectedValues = Array.from(event.target.selectedOptions).map(
                      (option) => option.value,
                    );
                    setValues((current) => ({
                      ...current,
                      [field.name]: selectedValues.join(","),
                    }));
                    return;
                  }

                  setValues((current) => ({ ...current, [field.name]: event.target.value }));
                }}
              >
                {field.options?.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            ) : (
              <input
                className="input"
                value={values[field.name] ?? ""}
                onChange={(event) =>
                  setValues((current) => ({ ...current, [field.name]: event.target.value }))
                }
                placeholder={field.placeholder}
              />
            )}
          </label>
        ))}

        {confirmToken ? (
          <label>
            <span>{tokenLabel}</span>
            {tokenHint ? <p>{tokenHint}</p> : null}
            <code className="confirm-token">{confirmToken}</code>
            <input
              className="input"
              value={tokenInput}
              onChange={(event) => setTokenInput(event.target.value)}
              placeholder={tokenPlaceholder}
            />
          </label>
        ) : null}

        {validationError ? <p className="error-text">{validationError}</p> : null}
        {errorMessage ? <p className="error-text">{errorMessage}</p> : null}
        {running ? <LoadingIndicator size="sm" label={runningLabel} /> : null}

        <div className="row gap-sm">
          <button type="button" className="btn secondary" onClick={onCancel} disabled={running}>
            {cancelLabel}
          </button>
          <button
            type="button"
            className={danger ? "btn danger" : "btn"}
            onClick={() => {
              void submit();
            }}
            disabled={running}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

function parseCommaSeparated(raw: string): string[] {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);
}
