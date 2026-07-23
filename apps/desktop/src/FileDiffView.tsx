import type { FileDiff } from "@gitnova/protocol";

export function FileDiffView({ diff, emptyMessage = "No textual changes in this comparison." }: { diff: FileDiff; emptyMessage?: string }) {
  if (diff.isBinary) return <p className="empty-state">Binary file changed. Content is not returned by Core.</p>;
  if (diff.hunks.length === 0) return <p className="empty-state">{emptyMessage}</p>;
  return diff.hunks.map((hunk, hunkIndex) => (
    <section className="diff-hunk" key={`${hunk.oldStart}:${hunk.newStart}:${hunkIndex}`} aria-label={`Diff hunk ${hunkIndex + 1}`}>
      <h3>{`@@ -${hunk.oldStart},${hunk.oldLines} +${hunk.newStart},${hunk.newLines} @@${hunk.header ? ` ${hunk.header}` : ""}`}</h3>
      <ol>
        {hunk.lines.map((line, lineIndex) => (
          <li className={`diff-line diff-line--${line.kind}`} key={`${lineIndex}:${line.oldLine}:${line.newLine}`}>
            <span aria-label="Old line">{line.oldLine ?? ""}</span>
            <span aria-label="New line">{line.newLine ?? ""}</span>
            <span className="diff-line__prefix" aria-hidden="true">{line.kind === "addition" ? "+" : line.kind === "deletion" ? "−" : " "}</span>
            <code>{line.content}</code>
          </li>
        ))}
      </ol>
    </section>
  ));
}
