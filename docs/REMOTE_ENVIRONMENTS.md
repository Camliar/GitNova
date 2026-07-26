# Remote Core Environments

GitNova keeps one invariant across Desktop and future IDE Hosts: the repository path, System Git, Provider credentials and Core process belong to the same environment. Desktop remains local UI and byte transport; it does not mount, copy or interpret remote repositories.

## Launch targets

| Target | Direct process projection | Required setup |
| --- | --- | --- |
| This computer | bundled `gitnova-core` sidecar | installed Desktop bundle |
| WSL | `wsl.exe --distribution <name> --exec gitnova-core` | Windows with named distribution and compatible Core on its PATH |
| Remote SSH | `ssh -T -o BatchMode=yes -o ConnectTimeout=10 -- <destination> gitnova-core` | existing SSH config/key and compatible remote Core on PATH |
| Dev Container | `devcontainer exec --workspace-folder <absolute-local-folder> gitnova-core` | Dev Container CLI, running/resolvable container and Core inside it |

The strings above describe argument arrays, not shell commands. Distribution and destination accept only conservative identifier characters; workspace folder must use absolute POSIX or Windows drive syntax and contain no control characters. Validation is lexical and host-independent so a Windows runner can safely project both native and container-oriented paths. Remote Core executable name is fixed. A running Core locks its environment until shutdown.

## Repository paths

For local Core, Desktop uses the native directory picker. For WSL, SSH or Dev Container, the user enters the path as seen by that Core, such as `/home/user/project` or `/workspaces/project`. Desktop sends it opaquely to `repository/open`; it neither checks the local filesystem nor rewrites separators.

## Security and failures

SSH is non-interactive and honors the user's existing SSH configuration and host-key policy. GitNova does not collect passwords, private keys or remote tokens. WSL and Dev Container use their installed command-line launchers. stdout stays reserved for framed protocol data; child stderr is drained and not displayed. Missing tools, authentication failures, incompatible Core versions and timeouts return fixed Desktop lifecycle errors without command output.

There is no GitNova relay, remote daemon, TCP listener, port forwarding, repository upload or background reconnection.
