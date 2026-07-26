# Squash Trace Relationship

`github/squashTrace` is an explicit Core-owned network request that combines a normalized GitHub pull request with read-only local Git topology. It associates the PR and ordered original commits with GitHub's final merge commit OID while keeping Provider facts separate from Core inference.

The request accepts the same positive `number` and optional `remote` / `nameWithOwner` selectors as `github/pullRequest`. Core first obtains the complete PR and original commit sequence under the same 250-commit protection. It never fetches missing objects, writes the repository, or asks a Host to interpret Git or GitHub data.

## Relationship model

The result includes `classification`, `confidence`, `mergeCommitOid`, local availability, local parent OIDs, and ordered machine-readable evidence:

- `notMerged` / `high`: GitHub says the PR is not merged. Any test merge OID from an open PR is not presented as the final commit.
- `originalCommit` / `high`: the final merge OID exactly matches an original commit OID.
- `mergeCommit` / `high`: the final merge OID is distinct and its locally available commit has at least two parents.
- `squashCandidate` / `medium`: the final merge OID is distinct and its local commit has at most one parent.
- `unresolved` / `none`: GitHub omitted the final OID or the object is not available in the opened local repository.

`squashCandidate` is deliberately not a confirmed squash classification. GitHub's PR response does not provide the merge strategy, and a distinct single-parent result can also arise from a rebase workflow. Evidence therefore includes `providerMergeStrategyUnavailable`, allowing every Host to present the same honest explanation.

## Local-first behavior

Local topology inspection uses System Git `cat-file -e` and `rev-list --parents --max-count=1` against the already opened repository. A missing final commit is normal result data (`localCommitMissing`), not an automatic network fetch or request failure. Git unavailable, unsafe repository ownership, malformed commit output, and other execution failures retain the existing stable Git/repository errors. No commit content, patch, stderr, credentials, or raw Provider response is returned.

Original commit file and line details remain available through [`github/pullRequestCommitDiff`](GITHUB_PULL_REQUESTS.md).

## History-integrated discovery

Protocol 1.17 adds `github/commitSquashTrace` for the All Commits workflow. It accepts a full local commit OID and performs an explicit GitHub association request. Core keeps only merged PRs whose Provider `merge_commit_sha` exactly equals that OID. No exact match returns `trace: null` and leaves the ordinary local Commit/Changes detail unchanged; more than one exact match returns `github.commit_association_ambiguous` rather than choosing one.

When Core classifies a trace as `squashCandidate`, Desktop first shows the ordered PR original commits and the `originals → final commit` relationship. `mergeCommit`, `originalCommit` and unresolved results remain in the ordinary local Commit/Changes presentation; Desktop uses Core's classification and performs no topology inference. Selecting a squash candidate's original commit calls `github/pullRequestCommitFiles`, which validates PR membership and returns file metadata without patch hunks. The bounded full Provider response stays only in the repository-local Core memory cache. Selecting one file then calls `github/pullRequestCommitFileDiff`, which returns only that cached file. A different PR/commit selector replaces the cache, and switching repository environments starts a distinct Core process.

The association endpoint, PR detail, original commits, and commit files are all network access and run only after the user chooses **Check Squash Trace** or selects an original commit. There is no repository-open request, background retry, or Host-side Provider call.
