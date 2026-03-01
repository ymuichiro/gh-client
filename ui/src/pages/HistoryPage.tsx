import { useMemo, useState } from "react";

import { clearHistory, loadHistory } from "../core/history";
import { useI18n } from "../core/i18n";

export function HistoryPage(): JSX.Element {
  const { t } = useI18n();
  const [version, setVersion] = useState(0);

  const history = useMemo(() => {
    void version;
    return loadHistory();
  }, [version]);

  return (
    <section className="page-section">
      <header className="section-header">
        <h2>{t("history.title")}</h2>
        <button
          type="button"
          className="btn secondary"
          onClick={() => {
            clearHistory();
            setVersion((v) => v + 1);
          }}
        >
          Clear
        </button>
      </header>

      {history.length === 0 ? (
        <p>{t("history.empty")}</p>
      ) : (
        <table className="repo-table">
          <thead>
            <tr>
              <th>timestamp</th>
              <th>command_id</th>
              <th>request_id</th>
              <th>repo</th>
              <th>status</th>
              <th>code</th>
            </tr>
          </thead>
          <tbody>
            {history.map((entry) => (
              <tr key={`${entry.timestamp}-${entry.requestId}`}>
                <td>{entry.timestamp}</td>
                <td>{entry.commandId}</td>
                <td>{entry.requestId}</td>
                <td>{entry.repo ?? ""}</td>
                <td>{entry.status}</td>
                <td>{entry.code ?? ""}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
