import { useMemo, useState } from "react";

import { useI18n } from "../core/i18n";

interface ConfirmModalProps {
  open: boolean;
  commandId: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmModal({ open, commandId, onConfirm, onCancel }: ConfirmModalProps): JSX.Element | null {
  const { t } = useI18n();
  const [tokenInput, setTokenInput] = useState("");

  const expected = useMemo(() => `CONFIRM:${commandId}`, [commandId]);
  if (!open) {
    return null;
  }

  const matches = tokenInput.trim() === expected;

  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true">
      <div className="modal-panel">
        <h3>{t("confirm.title")}</h3>
        <p>{t("confirm.step1")}</p>
        <p>{t("confirm.step2")}</p>
        <code className="confirm-token">{expected}</code>
        <input
          className="input"
          value={tokenInput}
          onChange={(event) => setTokenInput(event.target.value)}
          placeholder={t("confirm.placeholder")}
        />
        {!matches && tokenInput.length > 0 ? <p className="error-text">{t("confirm.invalid")}</p> : null}
        <div className="row gap-sm">
          <button type="button" className="btn secondary" onClick={onCancel}>
            {t("common.cancel")}
          </button>
          <button type="button" className="btn danger" disabled={!matches} onClick={onConfirm}>
            {t("common.execute")}
          </button>
        </div>
      </div>
    </div>
  );
}
