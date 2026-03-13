import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { I18nProvider } from "../core/i18n";
import { executeCommand } from "../core/executor";
import { RepositorySelectionPage } from "./RepositorySelectionPage";

vi.mock("../core/executor", () => ({
  executeCommand: vi.fn(),
}));

describe("RepositorySelectionPage", () => {
  const executeCommandMock = vi.mocked(executeCommand);
  const originalRequestAnimationFrame = window.requestAnimationFrame;
  const originalCancelAnimationFrame = window.cancelAnimationFrame;

  beforeEach(() => {
    executeCommandMock.mockReset();
    vi.stubGlobal("requestAnimationFrame", ((callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    }) as typeof window.requestAnimationFrame);
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
  });

  afterEach(() => {
    window.requestAnimationFrame = originalRequestAnimationFrame;
    window.cancelAnimationFrame = originalCancelAnimationFrame;
    vi.unstubAllGlobals();
  });

  it("refreshes stale persisted repositories on manual update", async () => {
    executeCommandMock.mockResolvedValue({
      requestId: "req-1",
      commandId: "repo.list",
      payload: { owner: "acme", limit: 100 },
      data: [{ nameWithOwner: "acme/fresh-repo", viewerPermission: "viewer" }],
    } as never);

    render(
      <I18nProvider>
        <MemoryRouter>
          <RepositorySelectionPage
            initialConfig={{
              orgs: ["acme"],
              repositories: [
                {
                  owner: "acme",
                  repo: "stale-repo",
                  viewerPermission: "viewer",
                },
              ],
              updatedAt: "2020-01-01T00:00:00.000Z",
            }}
            onApplyConfig={vi.fn()}
          />
        </MemoryRouter>
      </I18nProvider>,
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: /Update repositories from checked organizations|チェック済み org から repo 候補を更新/,
      }),
    );

    await waitFor(() => {
      expect(executeCommandMock).toHaveBeenCalledWith(
        "repo.list",
        { owner: "acme", limit: 100 },
        { permission: "viewer" },
      );
    });

    await waitFor(() => {
      expect(screen.getByText("fresh-repo")).toBeInTheDocument();
    });
  });
});
