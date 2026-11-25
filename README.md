# Glues Minimal

A Vim-inspired terminal note-taking app with Instant, Local, and Git storage. Notes support Markdown, including tables, lists, links, and code blocks.

## Download

Download a ready-to-run binary from the [GitHub Releases](https://github.com/eqan/glues-minimal-terminal-note-taker/releases) page. Tagged releases publish binaries for Linux x86_64, macOS ARM64, and Windows x86_64.

## Build and run

Install Rust 1.90 or newer, then build from a checkout:

```bash
git clone https://github.com/eqan/glues-minimal-terminal-note-taker.git
cd glues-minimal-terminal-note-taker
cargo build --release -p glues
./target/release/glues
```

You can also install the executable directly from Git:

```bash
cargo install --git https://github.com/eqan/glues-minimal-terminal-note-taker.git glues
glues
```

## Storage

- **Instant** keeps notes in memory until the app exits.
- **Local** stores notes as files in the selected directory.
- **Git** stores notes in an existing Git repository and syncs through its configured remote and branch.

The last-used paths, Git settings, theme, and application log are saved under `~/.glues/`.

## Shortcuts

- `Ctrl+P` toggles Markdown preview in the note pane.
- `Ctrl+O` or `Ctrl+\` toggles the side pane. Either shortcut hides or restores it.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

## License

Apache-2.0. See [LICENSE](LICENSE).
