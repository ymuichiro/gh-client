import { useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import type { PropsWithChildren } from "react";

import { useI18n } from "../core/i18n";

interface ShellProps extends PropsWithChildren {}
type ThemeMode = "light" | "dark";
const THEME_STORAGE_KEY = "gh-client-theme-mode";

const navItems = [
  { to: "/issues", key: "nav.issues" },
  { to: "/pull-requests", key: "nav.pull_requests" },
  { to: "/settings", key: "nav.settings" },
] as const;

export function Shell({
  children,
}: ShellProps): JSX.Element {
  const { t } = useI18n();
  const [theme, setTheme] = useState<ThemeMode>(() => resolveInitialTheme());

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    const storage = getSafeLocalStorage();
    storage?.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

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

        <div className="sidebar-footer">
          <p className="sidebar-footer-label">{t("theme.label")}</p>
          <div className="theme-switch" role="group" aria-label={t("theme.label")}>
            <button
              type="button"
              className={theme === "light" ? "theme-chip active" : "theme-chip"}
              onClick={() => setTheme("light")}
            >
              {t("theme.light")}
            </button>
            <button
              type="button"
              className={theme === "dark" ? "theme-chip active" : "theme-chip"}
              onClick={() => setTheme("dark")}
            >
              {t("theme.dark")}
            </button>
          </div>
        </div>
      </aside>

      <div className="main-area">
        <main className="content">{children}</main>
      </div>
    </div>
  );
}

function resolveInitialTheme(): ThemeMode {
  const storage = getSafeLocalStorage();
  const stored = storage?.getItem(THEME_STORAGE_KEY);
  if (stored === "light" || stored === "dark") {
    return stored;
  }

  if (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  ) {
    return "dark";
  }

  return "light";
}

function getSafeLocalStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  const storage = window.localStorage;
  if (
    storage &&
    typeof storage.getItem === "function" &&
    typeof storage.setItem === "function"
  ) {
    return storage;
  }

  return null;
}
