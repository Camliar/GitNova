using GitNova.VisualStudio.Transport;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.VisualStudio.Extensibility;

namespace GitNova.VisualStudio;

[VisualStudioContribution]
public sealed class GitNovaExtension : Extension
{
    public override ExtensionConfiguration ExtensionConfiguration => new()
    {
        Metadata = new(
            id: "GitNova.4ea6fdd4-a7a2-4bc0-b847-6b9e34a29f40",
            version: ExtensionAssemblyVersion,
            publisherName: "GitNova",
            displayName: "GitNova",
            description: "Local-first Squash Trace for Visual Studio"),
    };

    protected override void InitializeServices(IServiceCollection services)
    {
        base.InitializeServices(services);
        services.AddSingleton<GitNovaCoreService>();
    }
}
