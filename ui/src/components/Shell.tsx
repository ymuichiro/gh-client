import { NavLink } from "react-router-dom";
import type { PropsWithChildren } from "react";

import { useI18n } from "../core/i18n";
import { LanguageSwitch } from "./LanguageSwitch";

interface ShellProps extends PropsWithChildren {
  owner: string;
  repo: string;
  onOwnerChange: (value: string) => void;
  onRepoChange: (value: string) => void;
  onRepoPermissionChange: (value: "viewer" | "write" | "admin" | "unknown") => void;
  repoPermission: string;
  authLoggedIn: boolean;
}

const navItems = [
  { to: "/", key: "nav.dashboard" },
  { to: "/repositories", key: "nav.repositories" },
  { to: "/pull-requests", key: "nav.pull_requests" },
  { to: "/issues", key: "nav.issues" },
  { to: "/actions", key: "nav.actions" },
  { to: "/releases", key: "nav.releases" },
  { to: "/settings", key: "nav.settings" },
  { to: "/p2", key: "nav.p2" },
  { to: "/console", key: "nav.console" },
  { to: "/history", key: "nav.history" },
] as const;

export function Shell({
  owner,
  repo,
  onOwnerChange,
  onRepoChange,
  onRepoPermissionChange,
  repoPermission,
  authLoggedIn,
  children,
}: ShellProps): JSX.Element {
  const { t } = useI18n();

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <h1>{t("app.title")}</h1>
          <p>{t("app.subtitle")}</p>
        </div>

        <nav className="nav">
          {navItems.map((item) => (
            <NavLink key={item.to} to={item.to} className={({ isActive }) => (isActive ? "nav-item active" : "nav-item")}>
              {t(item.key)}
            </NavLink>
          ))}
        </nav>
      </aside>

      <div className="main-area">
        <header className="topbar">
          <div className="context-grid">
            <label>
              <span>{t("header.owner")}</span>
              <input className="input" value={owner} onChange={(event) => onOwnerChange(event.target.value)} />
            </label>
            <label>
              <span>{t("header.repo")}</span>
              <input className="input" value={repo} onChange={(event) => onRepoChange(event.target.value)} />
            </label>
            <label>
              <span>{t("header.permission")}</span>
              <select
                className="input"
                value={repoPermission}
                onChange={(event) =>
                  onRepoPermissionChange(
                    event.target.value as "viewer" | "write" | "admin" | "unknown",
                  )
                }
              >
                <option value="unknown">unknown</option>
                <option value="viewer">viewer</option>
                <option value="write">write</option>
                <option value="admin">admin</option>
              </select>
            </label>
          </div>

          <div className="status-row">
            <div className="badge">
              {t("header.permission")}: <strong>{repoPermission}</strong>
            </div>
            <div className={authLoggedIn ? "badge success" : "badge danger"}>
              {t("header.auth")}: {authLoggedIn ? t("auth.logged_in") : t("auth.logged_out")}
            </div>
            <LanguageSwitch />
          </div>
        </header>

        <main className="content">{children}</main>
      </div>
    </div>
  );
}
