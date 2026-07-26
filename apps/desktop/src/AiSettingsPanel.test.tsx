import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AiSettingsPanel, defaultAiAssistSettings } from "./AiSettingsPanel";

describe("AI settings", () => {
  it("offers the supported providers and explains the Core-only credential boundary", () => {
    const onChange = vi.fn();
    const { rerender } = render(<AiSettingsPanel settings={defaultAiAssistSettings} onChange={onChange} />);
    const provider = screen.getByLabelText("Provider");
    expect(provider).toHaveTextContent("Ollama (local)");
    expect(provider).toHaveTextContent("OpenAI");
    expect(provider).toHaveTextContent("Claude (Anthropic)");
    expect(provider).toHaveTextContent("DeepSeek");
    expect(provider).toHaveTextContent("Qwen (Alibaba Cloud)");
    expect(provider).toHaveTextContent("Kimi (Moonshot AI)");

    fireEvent.change(provider, { target: { value: "anthropic" } });
    expect(onChange).toHaveBeenLastCalledWith({ ...defaultAiAssistSettings, providerKind: "anthropic" });
    rerender(<AiSettingsPanel settings={{ ...defaultAiAssistSettings, providerKind: "anthropic" }} onChange={onChange} />);
    expect(screen.getByText("ANTHROPIC_API_KEY")).toBeInTheDocument();
    expect(screen.queryByLabelText("Ollama loopback URL")).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/API key/i)).not.toBeInTheDocument();
  });
});
