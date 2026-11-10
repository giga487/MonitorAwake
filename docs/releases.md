# Windows releases

Public downloads are available on [GitHub Releases](https://github.com/giga487/MonitorAwake/releases), with a permanent [latest release link](https://github.com/giga487/MonitorAwake/releases/latest).

**Release Windows setup** (`.github/workflows/release.yml`) runs on a `vX.Y.Z` tag. It checks matching tag, Cargo, Tauri and npm versions, runs frontend/Rust checks, builds the Windows x64 NSIS installer, and attaches it to a draft release. The release is published only after verifying the setup asset exists. It uses the workflow's `GITHUB_TOKEN` with `contents: write`; no additional secret is required.

## Release a version

1. Update `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json` and both lockfiles together. Data schema versions are separate.
2. Run checks, commit and push.
3. Create and push an annotated tag matching the version. For the current version:

```sh
git tag -a v0.2.0 -m "MonitorAwake v0.2.0"
git push origin v0.2.0
```

4. Wait for **Release Windows setup** in Actions. The setup appears under the release's **Assets**.

To repeat a build, use **Actions > Release Windows setup > Run workflow** with an existing tag. Do not move published tags to different commits.

Release automation currently produces Windows setup; Linux compilation remains in the check workflow. Personal Firebase configuration must stay outside release assets. A version bump alone does not commit, tag or publish a release.

References: [Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/), [tauri-action](https://github.com/tauri-apps/tauri-action).
