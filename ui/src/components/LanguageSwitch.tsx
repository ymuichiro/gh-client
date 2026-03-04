import { useI18n } from "../core/i18n";

export function LanguageSwitch(): JSX.Element {
  const { language, setLanguage } = useI18n();

  return (
    <div className="language-switch" role="group" aria-label="language switch">
      <button
        type="button"
        className={language === "ja" ? "chip active" : "chip"}
        onClick={() => setLanguage("ja")}
      >
        JA
      </button>
      <button
        type="button"
        className={language === "en" ? "chip active" : "chip"}
        onClick={() => setLanguage("en")}
      >
        EN
      </button>
    </div>
  );
}
