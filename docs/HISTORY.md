# Paginated Commit History

`repository/history` returns read-only commit history for the repository opened in the current Core session. It supports worktrees, linked worktrees, detached HEAD, and bare repositories.

## Request and page

Parameters are optional: `limit` defaults to 50 and must be between 1 and 200; `cursor` is an opaque string previously returned by Core. A page contains `commits` and nullable `nextCursor`. An unborn repository returns an empty page.

Each commit contains its OID, ordered parent OIDs, author and committer identities, the first message line as `summary`, and the complete message. Identity timestamps preserve the numeric timezone stored by Git and are serialized as ISO-8601. Metadata and messages must be UTF-8.

## Snapshot pagination

The first request resolves HEAD once. Its cursor binds subsequent pages to that commit and an offset, so commits created after page one do not shift or enter that sequence. Hosts must not parse, modify, persist as a revision identifier, or synthesize cursors; they only return `nextCursor` to Core.

Core obtains ordered OIDs with System Git `rev-list --topo-order --date-order`, then reads each raw object with `git cat-file commit`. It does not parse human-oriented `git log` output or use message delimiters.

An invalid, malformed, or unavailable cursor returns `history.invalid_cursor`. An invalid commit object returns `git.commit_parse_failed`; unsupported metadata/message encoding returns `history.unsupported_encoding`.

## Deliberate limits

The method follows ancestry reachable from the captured HEAD. It does not return file changes, patches, graph lanes, decorations, arbitrary ranges, reflog entries, or unreachable objects. Those capabilities require separate Tasks.

## Desktop history

Desktop 通过 [`repository/graph`](COMMIT_GRAPH.md) 消费相同的固定 HEAD 分页序列，以同时获得 Core-owned HEAD/ref decorations。每页固定请求 30 个 commits；`nextCursor` 只原样返回 Core，不解析、合成或持久化。

第一页面向所有 repository kind，包括 bare、detached 和 empty/unborn。Load more 只在用户明确触发时执行，失败会保留已加载 commits 与 cursor 以便重试。working-tree status Refresh 不重建 history snapshot；同仓库 reopen 会建立新的第一页 snapshot。
