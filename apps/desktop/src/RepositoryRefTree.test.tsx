import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryRefTree } from "./RepositoryRefTree";

const oid = "a".repeat(40);
const references = {
  head: { oid, symbolicRef: "refs/heads/main" },
  references: [
    { name: "main", fullName: "refs/heads/main", kind: "localBranch" as const, targetOid: oid, peeledTargetOid: null, symbolicTarget: null, upstream: "origin/main" },
    { name: "topic", fullName: "refs/heads/topic", kind: "localBranch" as const, targetOid: oid, peeledTargetOid: null, symbolicTarget: null, upstream: null },
    { name: "origin/HEAD", fullName: "refs/remotes/origin/HEAD", kind: "remoteBranch" as const, targetOid: oid, peeledTargetOid: null, symbolicTarget: "refs/remotes/origin/main", upstream: null },
    { name: "origin/topic", fullName: "refs/remotes/origin/topic", kind: "remoteBranch" as const, targetOid: oid, peeledTargetOid: null, symbolicTarget: null, upstream: null },
    { name: "v1.0", fullName: "refs/tags/v1.0", kind: "tag" as const, targetOid: oid, peeledTargetOid: oid, symbolicTarget: null, upstream: null },
  ],
};

describe("RepositoryRefTree", () => {
  it("offers exact local and remote checkout actions from mouse and keyboard-accessible triggers", () => {
    const onSwitch = vi.fn();
    const onCheckoutRemote = vi.fn();
    render(<RepositoryRefTree state={{ kind: "ready", value: references }} currentBranch="main" canSwitch canCheckoutRemote onSwitch={onSwitch} onCheckoutRemote={onCheckoutRemote} />);

    fireEvent.contextMenu(screen.getByTitle("Switch to topic"), { clientX: 20, clientY: 30 });
    fireEvent.click(within(screen.getByRole("menu", { name: "Branch actions for topic" })).getByRole("menuitem", { name: "Checkout" }));
    expect(onSwitch).toHaveBeenCalledWith("topic");

    fireEvent.click(screen.getByRole("button", { name: "Actions for origin/topic" }));
    fireEvent.click(within(screen.getByRole("menu", { name: "Branch actions for origin/topic" })).getByRole("menuitem", { name: "Checkout as local tracking branch" }));
    expect(onCheckoutRemote).toHaveBeenCalledWith("refs/remotes/origin/topic", "origin/topic");
    expect(screen.queryByRole("button", { name: "Actions for origin/HEAD" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Actions for v1.0" })).not.toBeInTheDocument();
  });

  it("marks the current branch and closes its disabled menu with Escape", () => {
    render(<RepositoryRefTree state={{ kind: "ready", value: references }} currentBranch="main" canSwitch canCheckoutRemote onSwitch={vi.fn()} onCheckoutRemote={vi.fn()} />);
    const current = screen.getByLabelText("Current branch main");
    expect(current).toHaveClass("is-current");
    fireEvent.contextMenu(current);
    expect(screen.getByRole("menuitem", { name: "Current branch" })).toBeDisabled();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
