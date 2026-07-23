import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

const core = vi.hoisted(() => ({
  getCoreStatus: vi.fn(),
  startCore: vi.fn(),
}));

vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  getCoreStatus: core.getCoreStatus,
  startCore: core.startCore,
}));

describe("Desktop foundation", () => {
  beforeEach(() => {
    core.getCoreStatus.mockResolvedValue({
      connected: false,
      protocolVersion: null,
      capabilities: null,
    });
    core.startCore.mockResolvedValue({
      connected: true,
      protocolVersion: "1.11",
      capabilities: null,
    });
  });

  it("renders a semantic, honest Core connection state", async () => {
    render(<App />);

    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
      "Understand the history behind the merge.",
    );
    expect(await screen.findByRole("button", { name: "Start Core" })).toBeEnabled();
    expect(screen.getByText("Core connection").parentElement).toHaveTextContent("Not running");
    expect(screen.queryByText(/connected/i)).not.toBeInTheDocument();
  });

  it("provides a keyboard skip target and a named brand link", () => {
    render(<App />);

    expect(screen.getByRole("link", { name: "GitNova home" })).toHaveAttribute(
      "href",
      "#main-content",
    );
    expect(document.getElementById("main-content")).toHaveAttribute("tabindex", "-1");
  });

  it("shows a sanitized retry state when Core startup fails", async () => {
    core.startCore.mockRejectedValue({
      code: "desktop.core_unavailable",
      message: "GitNova Core executable is unavailable",
      retryable: true,
    });
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Start Core" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "GitNova Core executable is unavailable. No repository data was changed.",
    );
    expect(screen.getByRole("button", { name: "Retry Core" })).toBeEnabled();
  });
});
