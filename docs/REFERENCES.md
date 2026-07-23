# Repository References

`repository/references` returns read-only HEAD state and the public branch/tag references from the repository opened in the current Core session. It accepts no parameters and supports worktrees, linked worktrees, detached HEAD, unborn repositories, and bare repositories.

## HEAD

`head.oid` is the resolved commit OID when HEAD exists. `head.symbolicRef` is the full local branch ref when HEAD is attached.

- Attached: both fields are present.
- Detached: only `oid` is present.
- Unborn branch: only `symbolicRef` is present.

## References

References are sorted by full refname and classified as `localBranch`, `remoteBranch`, or `tag`. Each item contains a short `name`, `fullName`, direct `targetOid`, nullable `peeledTargetOid`, nullable `symbolicTarget`, and nullable local-branch `upstream`.

For annotated tags, `targetOid` identifies the tag object and `peeledTargetOid` identifies its peeled target. Lightweight tags have no peeled field. Remote symbolic refs such as `refs/remotes/origin/HEAD` remain visible and preserve their full symbolic target.

Core uses System Git `symbolic-ref`, `rev-parse`, and sorted `for-each-ref` output with NUL-separated fields under `LC_ALL=C`. Invalid output returns `git.reference_parse_failed`; non-UTF-8 reference metadata returns `reference.unsupported_encoding`.

## Deliberate limits

Only `refs/heads`, `refs/remotes`, and `refs/tags` are returned. The method does not mutate refs, switch branches, configure upstreams, fetch/push, read reflogs, or expose stash, notes, replace, bisect, or other internal refs.
