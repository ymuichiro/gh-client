import { NavLink } from "react-router-dom";
import type { PropsWithChildren } from "react";

import { useI18n } from "../core/i18n";

interface ShellProps extends PropsWithChildren {}

const navItems = [
  { to: "/issues", key: "nav.issues" },
  { to: "/pull-requests", key: "nav.pull_requests" },
  { to: "/settings", key: "nav.settings" },
] as const;

export function Shell({
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
        <main className="content">{children}</main>
      </div>
    </div>
  );
}
