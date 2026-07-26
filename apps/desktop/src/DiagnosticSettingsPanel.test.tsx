import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DiagnosticSettingsPanel } from "./DiagnosticSettingsPanel";

const diagnostics = vi.hoisted(() => ({ getDiagnosticInfo: vi.fn() }));
vi.mock("./diagnostics", () => diagnostics);

describe("Diagnostic settings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    diagnostics.getDiagnosticInfo.mockResolvedValue({
      path: "/Users/test/Library/Logs/dev.gitnova.desktop/diagnostics.jsonl",
      maxBytes: 1024 * 1024,
      retainedFiles: 2,
    });
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
  });

  it("shows and copies the local log location", async () => {
    render(<DiagnosticSettingsPanel />);
    const path = await screen.findByText("/Users/test/Library/Logs/dev.gitnova.desktop/diagnostics.jsonl");
    expect(path).toBeInTheDocument();
    expect(screen.getByText(/keeps 2 files/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Copy path" }));
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(path.textContent);
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
  });

  it("does not invent a path outside the installed app", async () => {
    diagnostics.getDiagnosticInfo.mockResolvedValue(null);
    render(<DiagnosticSettingsPanel />);
    expect(await screen.findByText("The diagnostic path is available in the installed Desktop app.")).toBeInTheDocument();
  });
});
