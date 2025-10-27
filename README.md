# hyprconf

Tiny Rust TUI to skim-search your Hyprland configuration and open a file in your editor.

What it does:

- Scans: `hyprland.conf`, utilities (`hyprpaper.conf`, `hyprlock.conf`, `hypridle.conf`), `conf.d/*.conf`, `themes/*.conf`, `plugins/*.conf`, and executable `scripts/*`.
- Shows a compact, colored list: `[category] alias — short description | file (path)`.
- Opens the selected file in `$EDITOR` (fallback: `hx`).
- Honors `$XDG_CONFIG_HOME`; otherwise uses `~/.config/hypr` as the root.

Build/install:

- `cargo build --release`
- or `cargo install --path .`

Usage:

- `hyprconf` — scan the default root and open a picker
- `hyprconf --root DIR` — use a different Hypr config directory
- `hyprconf --category {hyprland,utility,themes,plugins,conf-d,scripts}` — pre-filter
- `hyprconf --editor CMD` — pick an editor (else `$EDITOR`, else `hx`)
- `hyprconf --color SPEC` — skim color theme (e.g. `dark`, `light`, or a custom spec)
- `hyprconf --no-seg-colors` — disable per-line segment coloring

Examples:

- `hyprconf`                   # default root (`$XDG_CONFIG_HOME/hypr` or `~/.config/hypr`)
- `hyprconf --category conf-d` # only show conf.d entries
- `hyprconf --editor nvim`     # open selection in neovim
- `hyprconf --root ~/.config/hypr` # explicit root

Notes:

- The first non-shebang comment line is used as the short description when present.
- No external fuzzy-finder binary is required; this uses the `skim` crate directly.
