# Threepole Custom – einfache Windows-MSI über GitHub bauen

Die ZIP enthält bereits den Windows-Build-Workflow. Du musst auf deinem PC weder Rust noch Visual Studio installieren.

## Einmalig
1. Erstelle auf GitHub ein **privates** Repository.
2. Entpacke diese Source-ZIP und lade den Inhalt des Ordners `threepole-1.1.2` in das Repository hoch. Wichtig: `.github/workflows/build-windows.yml` muss mit hochgeladen werden.
3. Erstelle einen Bungie-API-Key für deine eigene Bungie-Anwendung.
4. Öffne im GitHub-Repository: **Settings -> Secrets and variables -> Actions -> New repository secret**.
5. Name: `BUNGIE_API_KEY`
6. Value: dein Bungie-API-Key.

## MSI bauen
1. Öffne im Repository **Actions**.
2. Wähle **Build Threepole Custom for Windows**.
3. Klicke **Run workflow**.
4. Warte, bis der Lauf grün abgeschlossen ist.
5. Unten beim Lauf unter **Artifacts** `threepole-custom-windows-msi` herunterladen.
6. Die heruntergeladene Artifact-ZIP entpacken.
7. Die darin enthaltene `.msi` starten und Threepole Custom installieren.

## Hinweis zum API-Key
Threepole 1.1.2 bindet den Bungie-API-Key beim Kompilieren in die Anwendung ein. Deshalb Repository und Build-Artefakte am besten privat lassen und den fertigen Installer nicht öffentlich verteilen.

## Parallel zur Originalversion
Die Custom-Version hat eine eigene Windows-App-ID und einen eigenen Config-Ordner. Sie überschreibt die offizielle Threepole-Installation nicht. Profil und Einstellungen müssen deshalb in Threepole Custom einmal separat eingerichtet werden.
