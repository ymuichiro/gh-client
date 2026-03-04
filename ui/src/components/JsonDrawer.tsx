import { useI18n } from "../core/i18n";

interface JsonDrawerProps {
  open: boolean;
  title: string;
  value: unknown;
  onClose: () => void;
}

export function JsonDrawer({ open, title, value, onClose }: JsonDrawerProps): JSX.Element | null {
  const { t } = useI18n();
  if (!open) {
    return null;
  }

  return (
    <aside className="drawer">
      <div className="drawer-header">
        <h3>{title}</h3>
        <button type="button" className="btn secondary" onClick={onClose}>
          {t("common.close")}
        </button>
      </div>
      <pre className="json-block">{JSON.stringify(value, null, 2)}</pre>
    </aside>
  );
}
