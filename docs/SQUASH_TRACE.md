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

Original commit file and line details remain available through [`github/pullRequestCommitDiff`](GITHUB_PULL_REQUESTS.md). This Task supplies the relationship read model; Host visualization is a separate Task.
