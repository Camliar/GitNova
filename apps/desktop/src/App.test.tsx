import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("Desktop foundation", () => {
  it("renders a semantic, honest Core connection state", () => {
    render(<App />);

    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
      "Understand the history behind the merge.",
    );
    expect(screen.getByRole("status")).toHaveTextContent("connect this Host");
    expect(screen.getByText("Core connection").parentElement).toHaveTextContent("Next task");
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
});
