using GitNova.VisualStudio.Transport;
using Microsoft.VisualStudio.Extensibility;
using Microsoft.VisualStudio.Extensibility.Commands;
using Microsoft.VisualStudio.Extensibility.Shell;

namespace GitNova.VisualStudio;

[VisualStudioContribution]
public sealed class InspectPullRequestCommand : Command
{
    private readonly GitNovaCoreService core;

    public InspectPullRequestCommand(VisualStudioExtensibility extensibility, GitNovaCoreService core) : base(extensibility)
    {
        this.core = core;
        DisableDuringExecution = true;
    }

    public override CommandConfiguration CommandConfiguration => new("%GitNova.VisualStudio.InspectPullRequestCommand.DisplayName%")
    {
        Placements = [CommandPlacement.KnownPlacements.ExtensionsMenu],
        Icon = new(ImageMoniker.KnownValues.Extension, IconSettings.IconAndText),
        EnabledWhen = ActivationConstraint.SolutionState(SolutionState.Exists),
    };

    public override async Task ExecuteCommandAsync(IClientContext context, CancellationToken cancellationToken)
    {
        string? repositoryPath = await ResolveRepositoryPathAsync(context, cancellationToken);
        if (repositoryPath is null) return;
        string? numberText = await Extensibility.Shell().ShowPromptAsync(
            "Enter the GitHub pull request number. Core will contact the configured Provider only after confirmation.",
            InputPromptOptions.Default with { Title = "GitNova Squash Trace" }, cancellationToken);
        if (!int.TryParse(numberText, out int number) || number <= 0)
        {
            if (numberText is not null) await ShowAsync("Enter a positive pull request number.", cancellationToken);
            return;
        }

        try
        {
            SquashTraceResult traceOnly = await core.LoadTraceAsync(repositoryPath, number, null, cancellationToken);
            string? oid = await Extensibility.Shell().ShowPromptAsync(
                traceOnly.Render() + "\nEnter an original commit OID for its remote file/line diff, or leave blank for relationship only.",
                InputPromptOptions.Default with { Title = $"GitNova PR #{number}" }, cancellationToken);
            SquashTraceResult result = string.IsNullOrWhiteSpace(oid) ? traceOnly : await core.LoadTraceAsync(repositoryPath, number, oid.Trim(), cancellationToken);
            await ShowAsync(result.Render(), cancellationToken);
        }
        catch (Exception error) when (error is not OperationCanceledException)
        {
            await ShowAsync(error.Message, cancellationToken);
        }
    }

    private async Task<string?> ResolveRepositoryPathAsync(IClientContext context, CancellationToken cancellationToken)
    {
        var project = await context.GetActiveProjectAsync(cancellationToken);
        if (project?.Path is null)
        {
            await ShowAsync("Open or select a project inside the repository first.", cancellationToken);
            return null;
        }
        string? directory = Path.GetDirectoryName(project.Path);
        if (directory is null || !Path.IsPathFullyQualified(directory))
        {
            await ShowAsync("The active project path is not an absolute local path.", cancellationToken);
            return null;
        }
        return directory;
    }

    private async Task ShowAsync(string message, CancellationToken cancellationToken) =>
        await Extensibility.Shell().ShowPromptAsync(message, PromptOptions.OK, cancellationToken);
}
