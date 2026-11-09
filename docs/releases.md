# Release Windows

La pagina pubblica e [GitHub Releases](https://github.com/giga487/MonitorAwake/releases), con collegamento permanente [ultima versione](https://github.com/giga487/MonitorAwake/releases/latest).

Il workflow **Release Windows setup** (`.github/workflows/release.yml`) parte quando viene pubblicato un tag `vX.Y.Z`. Controlla che tag, Cargo, Tauri e npm abbiano la stessa versione, esegue controlli frontend e test Rust, compila il setup NSIS su Windows x64 e lo allega a una release in bozza. Pubblica la release solo dopo aver verificato la presenza del setup. Non richiede segreti aggiuntivi: usa `GITHUB_TOKEN` del workflow con `contents: write`.

Per una nuova versione:

1. Aggiornare insieme `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json` e i rispettivi lockfile. La versione dello schema dati e separata.
2. Eseguire controlli, commit e push.
3. Creare un tag annotato, per esempio `git tag -a v0.1.1 -m "MonitorAwake v0.1.1"`, e pubblicarlo con `git push origin v0.1.1`.
4. Attendere **Release Windows setup** in Actions. Il setup compare nella sezione **Assets** della release.

Per ripetere una build usare **Actions > Release Windows setup > Run workflow**, indicando un tag gia esistente. Non spostare tag gia pubblicati su commit diversi.

Il rilascio automatico comprende il setup Windows; la compilazione Linux resta nel workflow di controllo. Riferimenti: [Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/) e [tauri-action](https://github.com/tauri-apps/tauri-action).
