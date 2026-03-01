import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";

import App from "./App";
import { I18nProvider } from "./core/i18n";

describe("App", () => {
  it("renders main navigation", () => {
    const client = new QueryClient();

    render(
      <QueryClientProvider client={client}>
        <I18nProvider>
          <BrowserRouter>
            <App />
          </BrowserRouter>
        </I18nProvider>
      </QueryClientProvider>,
    );

    expect(screen.getByText("gh-client")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "ダッシュボード" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Command Console" })).toBeInTheDocument();
  });
});
