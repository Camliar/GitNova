package dev.gitnova.idea;

import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.intellij.openapi.actionSystem.AnAction;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.application.ApplicationManager;
import com.intellij.openapi.progress.ProgressIndicator;
import com.intellij.openapi.progress.ProgressManager;
import com.intellij.openapi.progress.Task;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.ui.DialogWrapper;
import com.intellij.openapi.ui.Messages;
import java.awt.Dimension;
import javax.swing.JComponent;
import javax.swing.JScrollPane;
import javax.swing.JTextArea;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

public final class InspectPullRequestAction extends AnAction {
    @Override
    public void actionPerformed(@NotNull AnActionEvent event) {
        Project project = event.getProject();
        if (project == null || project.getBasePath() == null) return;
        String value = Messages.showInputDialog(project, "Core will request this PR and any selected commit patch through your configured GitHub Provider.", "GitNova Squash Trace", null);
        if (value == null) return;
        int number;
        try {
            number = Integer.parseInt(value);
            if (number <= 0) throw new NumberFormatException();
        } catch (NumberFormatException error) {
            Messages.showErrorDialog(project, "Enter a positive pull request number.", "GitNova");
            return;
        }
        ProgressManager.getInstance().run(new LoadTraceTask(project, number));
    }

    private static final class LoadTraceTask extends Task.Backgroundable {
        private final int number;

        private LoadTraceTask(Project project, int number) {
            super(project, "Loading GitNova Squash Trace", true);
            this.number = number;
        }

        @Override
        public void run(@NotNull ProgressIndicator indicator) {
            try {
                CoreService core = getProject().getService(CoreService.class);
                JsonObject open = new JsonObject();
                open.addProperty("path", getProject().getBasePath());
                core.request("repository/open", open);
                JsonObject repository = core.request("github/repository", new JsonObject());
                JsonObject params = new JsonObject();
                params.addProperty("number", number);
                params.addProperty("nameWithOwner", repository.get("nameWithOwner").getAsString());
                JsonObject trace = core.request("github/squashTrace", params);
                ApplicationManager.getApplication().invokeLater(() -> chooseCommit(core, repository, trace));
            } catch (Exception error) {
                ApplicationManager.getApplication().invokeLater(() -> Messages.showErrorDialog(getProject(), safeMessage(error), "GitNova"));
            }
        }

        private void chooseCommit(CoreService core, JsonObject repository, JsonObject trace) {
            JsonArray commits = trace.getAsJsonObject("pullRequest").getAsJsonArray("commits");
            String[] labels = new String[commits.size()];
            for (int index = 0; index < commits.size(); index++) {
                JsonObject commit = commits.get(index).getAsJsonObject();
                labels[index] = commit.get("oid").getAsString().substring(0, 8) + "  " + commit.get("summary").getAsString();
            }
            int selected = Messages.showChooseDialog(getProject(), "Select an original commit to request its remote patch. Cancel shows relationship only.", "GitNova PR #" + number, null, labels, labels.length == 0 ? null : labels[0]);
            if (selected < 0) {
                new ResultDialog(getProject(), trace, null).show();
                return;
            }
            ProgressManager.getInstance().run(new Task.Backgroundable(getProject(), "Loading GitNova commit diff", true) {
                @Override
                public void run(@NotNull ProgressIndicator indicator) {
                    try {
                        JsonObject params = new JsonObject();
                        params.addProperty("number", number);
                        params.addProperty("nameWithOwner", repository.get("nameWithOwner").getAsString());
                        params.addProperty("oid", commits.get(selected).getAsJsonObject().get("oid").getAsString());
                        JsonObject diff = core.request("github/pullRequestCommitDiff", params);
                        ApplicationManager.getApplication().invokeLater(() -> new ResultDialog(getProject(), trace, diff).show());
                    } catch (Exception error) {
                        ApplicationManager.getApplication().invokeLater(() -> Messages.showErrorDialog(getProject(), safeMessage(error), "GitNova"));
                    }
                }
            });
        }
    }

    private static String safeMessage(Exception error) {
        String message = error.getMessage();
        return message == null || message.isBlank() ? "GitNova operation failed" : message;
    }

    private static final class ResultDialog extends DialogWrapper {
        private final String content;

        private ResultDialog(Project project, JsonObject trace, @Nullable JsonObject diff) {
            super(project);
            setTitle("GitNova Squash Trace");
            var gson = new GsonBuilder().setPrettyPrinting().create();
            content = gson.toJson(trace) + (diff == null ? "" : "\n\nSelected original commit diff\n" + gson.toJson(diff));
            init();
        }

        @Override
        protected @Nullable JComponent createCenterPanel() {
            JTextArea text = new JTextArea(content);
            text.setEditable(false);
            text.setLineWrap(false);
            JScrollPane pane = new JScrollPane(text);
            pane.setPreferredSize(new Dimension(760, 560));
            return pane;
        }
    }
}
