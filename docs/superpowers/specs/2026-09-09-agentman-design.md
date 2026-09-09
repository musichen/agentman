# Agentman v0.1 design

## Purpose

Agentman is a local, interactive Rust terminal UI for finding and managing
sessions created by the coding agents installed on a developer's machine. It
offers one fast workflow for session search, rename, resume, fork, YOLO launch,
and recoverable cleanup.

## Scope

The first release supports these local agents:

- Codex (`~/.codex`)
- Claude Code (`~/.claude`)
- OpenClaude (`~/.openclaude`)
- Pi (`~/.pi`)
- Codewhale (`~/.codewhale`)
- DSH / DeepSeek Harness (`~/.dsh`)
- ACRYL (`~/.acryl`)

It is macOS-first, because deletion uses the macOS Trash and the project is
released from macOS. It must not contact remote services during normal use.

## Architecture

One `agentman` binary uses Ratatui and Crossterm, matching the dependable
terminal model of `nvmman`: alternate screen, raw mode, a guard that restores
the terminal, mouse capture, wheel scrolling, colour, and keyboard control.

`AgentAdapter` is the only extension seam. Each built-in adapter discovers its
own storage, parses session metadata into the shared `Session` view model, and
knows how to rename, trash, resume, and fork it. An adapter first uses a native
agent command when that command has the needed operation; it otherwise performs
the equivalent storage-specific local change. Agentman never claims an operation
is generic merely because the UI looks generic.

The shared `Session` model contains: agent, stable ID, title, project/current
directory when discoverable, modified time, native file or directory path, and
capabilities. Parse failures appear as read-only records with the raw path and
an explanatory status rather than being hidden.

## Session operations

### Search and browse

The opening screen shows detected agents and their session counts. Selecting an
agent opens a sortable, wheel-scrollable session list. `/` focuses a case
insensitive fuzzy filter that matches title, ID, and project path. The list
refreshes with `r`.

### Rename

`r` opens an inline editor. The adapter changes the native session title only
when the format contains a native title field. Otherwise it creates a clearly
labelled Agentman local display-name sidecar keyed by the agent and stable
session ID; it never rewrites opaque conversation content to invent a title.
Writes are atomic: serialize to a sibling temporary file, fsync it, then rename
it over the original.

### Resume and fork

Enter resumes/launches the selected session and `f` forks it. Agentman restores
the terminal before starting an interactive child process, then waits for that
process and re-enters the UI. When an installed CLI exposes the operation,
Agentman constructs and runs its official command. Otherwise the adapter
duplicates/re-IDs its native session material only when that format makes a
safe fork possible; unsafe formats are reported as unsupported rather than
silently copied.

`y` resumes in YOLO mode and `Y` forks in YOLO mode. The adapter owns the exact
agent-specific flags. Codex uses
`--dangerously-bypass-approvals-and-sandbox`; Claude Code uses
`--dangerously-skip-permissions`. Other agents use their installed CLI's
verified equivalent where one exists. If no such switch exists, Agentman shows
the exact command and asks for confirmation; it never fabricates a YOLO flag.

### Delete

`d` opens a modal naming the agent and session. Only `y` or clicking **Move to
Trash** confirms. The adapter sends the exact native session file or directory
to the macOS Trash. Nothing is permanently removed; Finder can restore it.

## Interaction design

The palette follows the supplied Agentman logo: monochrome high-contrast
surfaces, cyan/blue status accents, geometric borders, and compact typography.
The repository and crate page carry the PNG logo; the TUI uses a dependable
text mark, not terminal-image protocol support.

Keyboard shortcuts are always visible in the footer:

- `↑` / `↓` or mouse wheel: navigate
- `Enter`: resume
- `f`: fork
- `y` / `Y`: YOLO resume / fork
- `r`: rename
- `d`: move to Trash
- `/`: search
- `?`: help
- `q`: back or quit

Clickable rows and action controls provide the matching mouse path. Mouse use
is additive; all functionality remains keyboard-accessible.

## Constraints and safety

- No secret, token, prompt, or session content is shown in the UI or logs.
- Native commands have precedence over direct storage changes.
- File edits are limited to recognized metadata fields and atomic rewrites.
- Trash is the only deletion mechanism.
- The shipped crate contains no user-specific paths or credentials.
- v0.1 has no background daemon, sync service, cloud API, plugin system, or
  agent autodownload. More agents can be added as adapters later.

## Tests and release

Fixture tests cover each adapter's discovery and metadata parsing, fuzzy search,
rename mutation, launch-command construction, fork capability decisions, and
trash target selection. The release gate is `cargo fmt --check`, Clippy with
warnings denied, and `cargo test`.

The deliverables are a public MIT repository at `github.com/musichen/agentman`,
a GitHub Actions Rust CI workflow, a README with installation and safety notes,
the supplied logo, a GitHub release, and a public `agentman` v0.1.0 crate on
crates.io. The crates.io token is read only at publish time from the user-named
secure JSON file and is never printed or committed.
