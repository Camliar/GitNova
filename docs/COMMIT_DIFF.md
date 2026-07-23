# Structured Commit Diff

`repository/commitDiff` returns commit metadata plus an ordered list of structured file diffs from the repository opened in the current Core session. It works in worktree, detached, and bare repositories and never reads working-tree state.

## Request

- `oid` is a required full 40- or 64-character hexadecimal commit OID.
- `parentOid` is optional. A root commit uses the empty-tree comparison and rejects a parent. A single-parent commit selects its only parent by default. A merge commit requires one of its direct parents explicitly.
- `contextLines` defaults to 3 and accepts 0–20.

Core verifies the raw commit object and direct parent relationship before diffing. Invalid syntax is `protocol.invalid_params`; a missing object is `commit.not_found`; an omitted merge parent is `commit.parent_required`; a non-parent is `commit.invalid_parent`.

## Result and implementation

The result contains `commit`, nullable `parentOid`, and `files`. Each file uses the same `FileDiff`, hunk, line-kind, and line-number contract as [`repository/diff`](DIFF.md), including rename, binary, and empty-file behavior.

Core invokes System Git directly. It obtains the ordered change list with NUL-delimited `--name-status -z`, then requests a patch for each literal repository path with external diff and text conversion disabled. It does not parse human-oriented `git log` output or invoke a shell.

## Deliberate limits

The method compares exactly one commit-parent edge. It does not produce combined merge diffs, choose a preferred merge parent, compare arbitrary trees/ranges, include working-tree changes, apply patches, or retrieve remote-only objects.
