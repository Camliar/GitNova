# ADR-0006: Remote Core Launching

- **Status:** Accepted
- **Date:** 2026-07-25

## Context

WSL, Remote SSH and Dev Container repositories do not necessarily share the Desktop Host's filesystem, System Git, `gh` authentication or path semantics. Running Core beside the Host and forwarding a remote path would violate “repository where Core runs”.

## Decision

Hosts may start Core through a structured environment launcher while preserving the exact JSON-RPC 2.0 stdio transport. The launched `gitnova-core` executable runs inside the selected repository environment and remains the only business capability layer.

Desktop supports four closed targets: bundled local sidecar, `wsl.exe --distribution … --exec gitnova-core`, batch-mode `ssh … gitnova-core`, and `devcontainer exec --workspace-folder … gitnova-core`. Inputs are validated typed fields and are passed as process arguments without a local shell. Arbitrary commands, arguments, port forwarding, credential copying, repository synchronization and remote daemons are prohibited.

Remote Core installation and authentication are user/environment responsibilities. The Host drains stderr, enforces the existing handshake/timeouts, and never translates repository paths between environments.

## Consequences

- Core observes the correct System Git, filesystem and Provider credentials.
- Existing protocol and business logic remain unchanged across local and remote environments.
- Users must install a protocol-compatible Core and supporting tools remotely before connecting.
- Each Host needs a small lifecycle adapter, but may not copy domain behavior.
