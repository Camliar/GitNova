# Desktop Packaging and Release

GitNova Desktop bundles the independently compiled `gitnova-core` executable as a Tauri external binary. The Host still starts it as a separate stdio process; packaging does not move business logic into Tauri.

## Local bundle

Run `npm run bundle:desktop`. The command builds Core in release mode, copies it to Tauri's target-qualified sidecar location, builds the frontend and creates the native bundles supported by the current OS. Generated sidecars and `target/` output are ignored.

Desktop bundle icons are generated from `assets/icons/gitnova-mark.svg`. The checked-in PNG, macOS ICNS and Windows ICO must be regenerated together so platform packaging cannot drift from the square brand source.

The macOS install smoke test must launch the app through Finder/LaunchServices rather than a terminal, open a GitHub repository, and run an explicit Connect GitHub or Squash Trace action with `gh` installed in `/opt/homebrew/bin` or `/usr/local/bin`. This verifies the packaged local Core PATH projection. Remote Core targets remain separate and must be tested with Provider CLI installed in that remote environment.

## CI and release boundary

- `.github/workflows/ci.yml` runs on `main`, pull requests and manual dispatch across macOS, Windows and Linux. Its token is read-only and it never references signing secrets.
- `.github/workflows/release.yml` runs only for a pushed `v*` tag. `check-release-tag.mjs` requires the tag to exactly equal `v` plus the Tauri version. It builds Linux x64 `.deb`/AppImage, Windows x64 NSIS/MSI and a macOS universal app/DMG into a draft GitHub Release.
- A maintainer reviews platform jobs, signature/notarization results and install smoke tests before manually publishing the draft. GitHub Releases is artifact distribution, not a GitNova server or account system.

## Signing credentials

Never commit certificates, private keys or passwords. Configure only the release environment/repository secrets:

| Platform | Secrets | Behavior when absent |
| --- | --- | --- |
| macOS | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH` | ad-hoc identity; artifact remains a draft and is not production-approved |
| Windows | `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` | installer is unsigned and must remain a draft; when present, the workflow imports the PFX and injects its thumbprint only for that job |
| Linux | distribution-specific signing credentials, added only after a separate policy decision | packages are unsigned; verify the immutable tag and artifact digest |

The release workflow deliberately does not invent or provision credentials. A missing or expired signature blocks publishing but does not block ordinary CI. Rotate credentials in GitHub settings and rerun the tag job; never copy secrets into logs or repository files.

## Version procedure

1. Update the same semantic version in the Desktop Tauri config and Rust/package manifests.
2. Run `npm run check`, `cargo test --workspace`, Clippy, rustfmt and `npm run bundle:desktop` on a supported native environment.
3. Merge the version change to `main`, then create and push the exact `vX.Y.Z` tag.
4. Inspect all draft artifacts and install/smoke test on native machines. Confirm Core starts from the installed application and the Squash Trace path works.
5. Publish the draft only when required platform signing and notarization policies pass.

No automatic updater, release daemon or central telemetry is introduced by this workflow.
