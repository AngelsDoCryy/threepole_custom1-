# Threepole Custom – Windows-Build und minimale lokale Umgebung

Stand der Bestandsaufnahme: 11.10.2026. Das vorhandene Repository ist `AngelsDoCryy/threepole_custom1-`, Entwicklungsbranch `optimize-speed-cache-icon`, Version 1.1.4. Kein neues Repository oder zusätzliches Dokumentationssystem ist erforderlich.

## Minimaler Weg: Codex lokal, Windows-Build über GitHub Actions

Für Dokumentation, Quelltextarbeit und die Nutzung des vorhandenen Windows-Workflows reichen Codex, Git, ein Checkout und GitHub-Zugriff. Rust und Visual Studio müssen dafür nicht lokal installiert sein. GitHub Actions läuft nur bei einem ausgelösten Build; Threepole benötigt dadurch keinen lokalen Hintergrunddienst und zur Laufzeit keine Verbindung zu Actions.

Der Workflow `.github/workflows/build-windows.yml` startet manuell sowie bei Pushes auf den Entwicklungsbranch, auch bei reinen Dokumentationsänderungen. Referenzumgebung: Node 20, Yarn 1.22.22, stable Rust, `windows-latest`. Das sind die verifizierten Workflow-Einstellungen, keine Aussage über den aktuellen Supportstatus dieser Toolversionen. Eine Toolchain-Modernisierung ist ein eigener Auftrag.

1. In **Actions → Build Threepole Custom for Windows → Run workflow** den richtigen Branch wählen, sofern ein manueller Build beauftragt ist.
2. Den erfolgreichen Lauf und dessen Commit prüfen.
3. Artifact `threepole-custom-windows-msi` herunterladen und entpacken.
4. Die MSI für die praktische Windows-Prüfung verwenden.

Der Workflow installiert Abhängigkeiten, führt die Overlay-Tests aus, baut die MSI, führt die Rust-Tests aus und lädt erst dann das Artifact hoch. `yarn tauri build` enthält bereits `yarn build`. Referenz: [Build #19](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/36518496000), Commit `fe083270b8a733239507f06bafde4048a879bfd7`; 37 Rust- und fünf Overlay-Tests bestanden. [MSI-Artifact](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/36518496000/artifacts/11011229394); Links können ablaufen. Neue Builds stets ihrem eigenen Commit zuordnen.

## Geprüfter lokaler Zustand

Die Prüfung war lesend; der vorhandene leere Git-Arbeitsordner wurde anschließend mit dem Entwicklungsbranch befüllt. Keine zusätzlichen Programme, Pakete oder Dienste wurden installiert.

| Bestandteil | Ergebnis am 11.10.2026 |
| --- | --- |
| Codex und Git | Verfügbar; Git 2.53.0.windows.3 aus der Codex-Laufzeit |
| Node | v24.19.0 aus der Codex-Laufzeit verfügbar; Projektkompatibilität damit noch nicht durch Build/Tests bestätigt |
| npm, Corepack, Yarn | Nicht im Suchpfad gefunden; der geprüfte Codex-Node-Ordner enthält nur node.exe |
| Rust/Cargo/rustup | Nicht im Suchpfad; Standardpfad im Benutzerprofil ebenfalls nicht vorhanden |
| MSVC/Build Tools/Windows SDK | Nicht im Suchpfad beziehungsweise an den geprüften Standardorten gefunden; keine systemweite Vollinventur |
| WebView2 | Laufzeit 154.0.4258.62 über Registry und Installationsordner nachgewiesen |

Ein teilweise eingeschränkter Registry-Zugriff wurde nicht als Beweis für fehlende Software gewertet. Ein vollständiger lokaler Windows-Build ist mit der nachgewiesenen Umgebung noch nicht möglich.

## Drei Bedarfsstufen

| Ziel | Tatsächlich benötigte Bestandteile |
| --- | --- |
| Dokumentation/Codeprüfung, fertige MSI aus Actions | Vorhandene Codex-App, Git/Checkout, GitHub-Verbindung; lokal keine Build-Toolchain nötig |
| Lokale Frontend-Tests und Vite-Build | Projektkompatibles Node, Yarn Classic (CI: 1.22.22), installierte Projektabhängigkeiten aus package.json/yarn.lock; kein Bungie-Key für reine Frontend-Tests |
| Lokale Rust-Tests, Desktop-App und MSI | Zusätzlich Rust/Cargo mit Windows-MSVC-Toolchain, Microsoft C++ Build Tools und Windows SDK; WebView2 zum Ausführen; BUNGIE_API_KEY als Build-Umgebungsvariable |

Die Tauri-CLI gehört bereits zu den Projektabhängigkeiten; eine zusätzliche globale Tauri-Installation ist nicht erforderlich. Für MSI verwendet Tauri WiX v3; die vorhandene CI installiert es nicht als eigenen Schritt, sondern lässt die Buildwerkzeuge durch den Tauri-Build bereitstellen. Initiale Builds können Tool-/Paketdownloads benötigen. Das sind Build-Abhängigkeiten, keine dauerhaft laufenden Dienste.

Ein vollständiges Visual-Studio-IDE-Paket, Docker, WSL, eine Datenbank, ein zusätzlicher MCP-Server oder eine separate Codex-CLI sind für diesen Projektweg nicht erforderlich. Der lokale Vite-Entwicklungsserver wäre nur eine bewusst gestartete Entwicklungsaufgabe und muss danach beendet werden; die installierte App verwendet die gebauten Dateien. WebView2 ist bereits vorhanden; keine Neuinstallation für diesen Auftrag.

Grundlagen: [Tauri-1-Windows-Voraussetzungen](https://v1.tauri.app/v1/guides/getting-started/prerequisites/) und [Windows-MSI-Build](https://v1.tauri.app/v1/guides/building/windows/). Vor einer späteren Einrichtung vorhandene Komponenten erneut prüfen und nur fehlende, freigegebene Bestandteile ergänzen.

## Prüf- und Buildbefehle für später eingerichtete Umgebungen

Diese Befehle wurden bei der Dokumentationsbereinigung **nicht** ausgeführt:

```powershell
yarn install
yarn build
node --test tests/overlay-visibility.test.mjs tests/overlay-activity.test.mjs
yarn tauri build
cargo test --release --manifest-path src-tauri/Cargo.toml --bin app
```

Die Reihenfolge Tauri vor Cargo-Tests entspricht dem Workflow, da Tauri die Features für den deaktivierten Updater konfiguriert. Änderungen an Manifest/Lockfiles durch Tools anschließend prüfen. Vite ersetzt keine separate TypeScript-/Svelte-Typprüfung; dafür gibt es aktuell keinen eigenen CI-Schritt.

Die MSI entsteht unter `src-tauri/target/release/bundle/msi/`. Das vorhandene `build-windows.ps1` kann Corepack aktivieren oder Yarn global installieren und führt nicht selbst die Regressionstests aus. Deshalb bei einem Auftrag „nichts installieren“ nicht einfach dieses Skript starten; Voraussetzungen zuerst prüfen und die beauftragten Einzelbefehle verwenden.

## API-Key und Verteilung

Der Workflow erwartet das vorhandene Repository-Secret `BUNGIE_API_KEY`; ein erfolgreicher früherer Build belegt dessen damalige Verfügbarkeit, nicht den aktuellen Secret-Zustand. Kein Key wurde bei der Bestandsaufnahme gelesen oder ausgegeben.

Rust verwendet `env!("BUNGIE_API_KEY")`: Der Wert muss beim Kompilieren vorhanden sein und wird in die Binärdatei eingebunden. Ein Secret schützt den Quelltext/Workflow-Wert, macht den Key im ausgelieferten Binary aber nicht geheim. Niemals den Key in Dokumentation, Quelltext, Logs oder Commit-Nachrichten schreiben.

Das Repository ist aktuell **öffentlich**; die alte Anleitung zur Erstellung eines privaten Repositorys beschreibt nicht den Ist-Zustand. Sichtbarkeit und Zugriff auf fertige Installer sind vor weiterer Verteilung bewusst zu behandeln. Dieser Auftrag ändert weder Repository-Sichtbarkeit noch Secrets oder Verteilungsrechte.

## Parallel zur Originalversion

Custom besitzt eine eigene Windows-App-ID, einen eigenen Konfigurationsordner und eine eigene Named Pipe. Profile/Einstellungen werden separat eingerichtet, der offizielle Updater ist deaktiviert. Installations-, Exit- und Live-Timing-Verhalten müssen trotzdem praktisch geprüft werden; siehe [PERFORMANCE](docs/PERFORMANCE.md).
