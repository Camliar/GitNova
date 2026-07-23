# Commit Graph Projection

`repository/graph` projects paginated commit history and repository references into one Core-owned read model. Hosts render that model but do not reproduce branch/tag-to-commit association logic.

## Request and pagination

The method accepts the same optional `limit` and opaque `cursor` as [`repository/history`](HISTORY.md). The default is 50 and the range is 1–200. Commit ordering and `nextCursor` use the same fixed-HEAD snapshot, so commits added after page one do not shift that sequence.

Each node contains a complete `CommitSummary`, `isHead`, and ordered `references`. Parent OIDs retain merge topology. Local and remote branches match their direct target OID. Annotated tags match their peeled target; lightweight tags match their direct target. Tags targeting a non-commit do not decorate a commit node.

References and `isHead` are deliberately read at each request rather than encoded into the history cursor. If refs move between pages, decorations reflect current repository state while the commit sequence remains fixed. Hosts should refresh the projection after ref-changing operations.

## Presentation boundary

Core owns Git semantics: commit ordering, parent edges, HEAD state, and reference decorations. Hosts own presentation mechanics such as lane routing, pixels, colors, animation, row virtualization, and responsive layout. Those presentation choices must consume Core data without invoking Git or reconstructing reference semantics.

The method does not search all refs, include unreachable commits, mutate branches/tags, fetch/push, or access a hosting provider.

## Desktop presentation

Desktop 使用 projection 的 node 顺序直接呈现 timeline，包括 commit summary、OID、author timestamp、parent count、`isHead` 和 references。Host 不自行关联 refs、剥离 annotated tags 或推断 HEAD。

Desktop 的 lane projector 是纯展示算法：first parent 延续当前 lane，额外 ordered parents 使用其他 lane，遇到已等待的 parent 时复用其 lane。分页追加后，Host 对全部已加载 nodes 重新计算视觉投影，使上一页末尾的 off-page parent continuation 与新节点连接。该算法只产生 SVG lane/connector 坐标，不改变 Core 顺序、parent 关系或 decoration 语义；窄屏可以裁剪非关键 lane，commit 文本、merge parent count、HEAD/ref 标签与可访问 graph 描述仍然保留。

分页使用固定 limit 30 和 Core opaque cursor。增量请求期间 Load more 被禁用；错误只影响该增量页面，已呈现 nodes 保持可用。repository reopen 会使旧请求失效并从无 cursor 的第一页重新加载，防止旧 snapshot 回写。

Timeline node 的 commit OID 与 ordered parents 同时作为 [`repository/commitDiff`](COMMIT_DIFF.md) 的唯一选择来源。Host 不缩短 OID 后再请求，也不从 decoration 或视觉顺序推导 parent。
