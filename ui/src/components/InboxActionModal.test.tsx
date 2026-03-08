import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { InboxActionModal } from "./InboxActionModal";

describe("InboxActionModal", () => {
  it("validates required field and confirmation token", async () => {
    const onConfirm = vi.fn(async () => undefined);

    render(
      <InboxActionModal
        open
        title="Batch close"
        description="confirm"
        fields={[{ name: "body", label: "Comment", type: "textarea", required: true }]}
        confirmLabel="Run"
        cancelLabel="Cancel"
        confirmToken="BATCH:CLOSE:2"
        tokenLabel="Token"
        tokenHint="type token"
        tokenPlaceholder="token-value"
        requiredFieldMessage={(label) => `${label} is required`}
        tokenMismatchMessage="Token mismatch"
        onCancel={() => undefined}
        onConfirm={onConfirm}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Run" }));
    expect(screen.getByText("Comment is required")).toBeInTheDocument();
    expect(onConfirm).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText("Comment"), {
      target: { value: "looks good" },
    });
    fireEvent.change(screen.getByPlaceholderText("token-value"), {
      target: { value: "wrong-token" },
    });

    fireEvent.click(screen.getByRole("button", { name: "Run" }));
    expect(screen.getByText("Token mismatch")).toBeInTheDocument();
    expect(onConfirm).not.toHaveBeenCalled();

    fireEvent.change(screen.getByPlaceholderText("token-value"), {
      target: { value: "BATCH:CLOSE:2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Run" }));

    await waitFor(() => expect(onConfirm).toHaveBeenCalledTimes(1));
    expect(onConfirm).toHaveBeenCalledWith({ body: "looks good" });
  });

  it("resets field values when reopened", () => {
    const { rerender } = render(
      <InboxActionModal
        open
        title="Comment"
        fields={[
          {
            name: "body",
            label: "Body",
            type: "textarea",
            initialValue: "default",
          },
        ]}
        confirmLabel="Send"
        cancelLabel="Cancel"
        requiredFieldMessage={(label) => `${label} is required`}
        tokenMismatchMessage="Token mismatch"
        onCancel={() => undefined}
        onConfirm={() => undefined}
      />,
    );

    fireEvent.change(screen.getByLabelText("Body"), {
      target: { value: "changed" },
    });

    rerender(
      <InboxActionModal
        open={false}
        title="Comment"
        fields={[
          {
            name: "body",
            label: "Body",
            type: "textarea",
            initialValue: "default",
          },
        ]}
        confirmLabel="Send"
        cancelLabel="Cancel"
        requiredFieldMessage={(label) => `${label} is required`}
        tokenMismatchMessage="Token mismatch"
        onCancel={() => undefined}
        onConfirm={() => undefined}
      />,
    );

    rerender(
      <InboxActionModal
        open
        title="Comment"
        fields={[
          {
            name: "body",
            label: "Body",
            type: "textarea",
            initialValue: "default",
          },
        ]}
        confirmLabel="Send"
        cancelLabel="Cancel"
        requiredFieldMessage={(label) => `${label} is required`}
        tokenMismatchMessage="Token mismatch"
        onCancel={() => undefined}
        onConfirm={() => undefined}
      />,
    );

    expect(screen.getByLabelText("Body")).toHaveValue("default");
  });
});
