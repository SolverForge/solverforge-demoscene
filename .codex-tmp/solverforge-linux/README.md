# SolverForge Linux

```bash
curl -sL https://raw.githubusercontent.com/pvd-dot/solverforge/main/boot.sh | bash
```

A layered configuration framework for an openSUSE Sway desktop. Immutable defaults ship in one directory, user overrides live in another, and a single orchestrator file wires everything into sway. No symlink farms, no dotfile managers, no magic.

**Version:** 1.1.0
**Platform:** openSUSE + Sway
**Shell:** zsh (primary), bash
**Terminal:** Kitty with Fira Code Nerd Font + Symbols Nerd Font

---

## Architecture

SolverForge Linux follows a two-layer model inspired by Rails-style convention over configuration:

```
~/.local/share/solverforge/          Framework root (immutable defaults)
    version                          Version file
    bin/                             All scripts (solverforge-* prefix)
    default/                         Default config layer
        sway/                        Compositor config (bindings, rules, output, input, autostart)
        wofi/                        Menu styling (CSS, banner SVG)
        kitty/                       Terminal mappings
        bash/                        Bash config modules
        zsh/                         Zsh config modules
        nvim/                        LazyVim plugin specs + extras preset
        waybar/                      Bar config, styling, logo SVG
        theme/                       Centralized theme system
        nerd-glyphs.json             Full Nerd Font icon database (10,764 glyphs)
        nerd-icons.txt               Searchable icon list

~/.config/solverforge/               User override layer
    backup.conf                      Backup credentials (real values)
    extensions/menu.sh               Custom menu entries
    tui-apps/                        User-installed TUI app configs
```

The orchestrator at `~/.config/sway/config.d/solverforge.conf` includes all default-layer sway configs and sets framework-wide variables (`$menu`, `$browser`). PATH is added via `~/.config/environment.d/solverforge.conf` for systemd user sessions.

Nothing is hardcoded to a specific user. CSS templates use `@@SOLVERFORGE_PATH@@` placeholders materialized at runtime. Tool paths (`$BROWSER`, `$EDITOR`, `$SWAYLOCK_CONFIG`) are resolved through environment variables with sensible defaults.

---

## Menu System

`solverforge-menu` is a hierarchical wofi-based launcher with a custom hexagonal ouroboros banner, openSUSE/copper color scheme, and verified Nerd Font icons on every entry.

```
Menu
  Apps              wofi --show drun with SolverForge styling
  TUI Apps          Keybindings, Wifi, Bluetooth, Disk Usage, Btop, LazyVim
                    + user-installed TUI apps from ~/.config/solverforge/tui-apps/
  Learn             Keybinding reference (solverforge-keys --dmenu)
  Capture
    Screenshot      Area/Full Screen to file or clipboard
    Color Picker    Pixel color to clipboard via ImageMagick
  Toggle            Waybar show/hide
  Setup             Audio, Monitors, Keybindings, Backup, Menu Extensions,
                    SolverForge config, LazyVim config
  Install           openSUSE Package (zypper+fzf), Web App, TUI App, LazyVim
  Remove            openSUSE Package, Web App, TUI App
  About             fastfetch with lolcat
  System            Lock, Suspend, Hibernate, Reboot, Shutdown
```

The menu CSS template lives in the default layer and is materialized to `$XDG_RUNTIME_DIR` at launch with the correct `$SOLVERFORGE_PATH` substituted, so it works for any user without hardcoded paths.

---

## Keybindings

All keybindings are defined in `default/sway/bindings.conf`. Mod is the Super key.

| Binding | Action |
|---|---|
| `Mod+Space` | App launcher (wofi drun with SolverForge styling) |
| `Mod+Alt+Space` | SolverForge hierarchical menu |
| `Mod+Escape` | System menu (lock/suspend/reboot/shutdown) |
| `Mod+k` | Keybinding guide (wofi dmenu) |
| `Mod+Shift+/` | Keybinding guide (terminal) |
| `Mod+c` | Copy (Ctrl+Insert via wtype) |
| `Mod+v` | Paste (Shift+Insert via wtype) |
| `Mod+x` | Cut (Ctrl+x via wtype) |
| `Mod+d` | Focus mode toggle |
| `Mod+\` | Split horizontal |
| `Mod+-` | Split vertical |
| `Mod+w` | Layout tabbed |
| `Mod+e` | Layout toggle split |
| `Mod+s` | Scratchpad show |
| `Mod+Shift+s` | Move to scratchpad |
| `Mod+Shift+w` | Layout stacking |
| `Mod+Shift+k` | Keystroke overlay toggle (wshowkeys) |
| `Mod+Shift+b` | Launch browser (`$browser`) |
| `Mod+Shift+m` | Launch Emacs client |
| `Mod+g` | Toggle Wayscriber screen annotation |
| `Super+Alt+k` | Cycle keyboard layout |

The unified copy/paste system uses `wtype` to send Ctrl+Insert / Shift+Insert, which works in both terminals and GUI apps without conflicting with Ctrl+C (SIGINT).

---

## Waybar

SolverForge Linux replaces the default openSUSEway waybar with a floating island bar. Hackerman aesthetic: semi-transparent dark background, rounded pill modules, neon teal/green glow on active states, and Nerd Font icons on every module.

### Layout

```
LEFT                                CENTER              RIGHT
[logo][workspaces][scratchpad]      [clock]             [cpu][mem][temp][disk][podman][ollama]
[voxtype][cava][language][audio]    [notifications]     [virsh][updates][net][bt][tray][power]
```

### Modules

| Module | Description |
|---|---|
| `image#solverforge` | Logo button, click opens SolverForge menu |
| `sway/workspaces` | Workspace indicators with teal glow on focused |
| `sway/scratchpad` | Scratchpad window count (hidden when empty) |
| `sway/mode` | Active sway mode (voxtype recording/suppress) |
| `sway/language` | Keyboard layout indicator |
| `custom/voxtype` | Voice typing status with recording pulse glow |
| `custom/cava` | Audio visualizer (requires `zypper install cava`) |
| `pulseaudio` | Volume, click mutes, right-click opens pavucontrol |
| `clock` | Time/date with calendar tooltip in teal/green |
| `custom/notifications` | SwayNC notification bell with pulse glow |
| `cpu` | CPU usage with warning (70%) and critical (90%) states |
| `memory` | RAM usage with warning/critical states |
| `temperature` | CPU temperature, critical at 80C |
| `disk` | Root partition usage |
| `custom/podman` | Running Podman container count (10s interval) |
| `custom/ollama` | Ollama loaded model count (10s interval) |
| `custom/virsh` | Running libvirt VM count (10s interval) |
| `custom/updates` | Pending zypper updates (1hr interval, click to install) |
| `network` | Ethernet/wifi status, right-click opens nmtui |
| `bluetooth` | Connection status, right-click opens bluetoothctl |
| `tray` | System tray (solaar, etc.) |
| `custom/power` | Click opens SolverForge system menu |

### Visual Design

- **Floating island** -- bar does not touch screen edges; rounded corners, subtle teal border glow
- **Pill modules** -- each module has its own rounded dark background with teal border
- **Neon glow** -- active workspace, recording state, and critical alerts get `box-shadow` glow effects
- **`@define-color` palette** -- entire color scheme defined as CSS variables at the top of `style.css`
- **Hover transitions** -- 0.3s ease-in-out on all modules
- **CPU-safe animations** -- all pulse effects use `steps()` timing to avoid render thrashing

### Wiring

```
~/.config/waybar/config    → symlink → default/waybar/config
~/.config/waybar/style.css → symlink → default/waybar/style.css
```

Waybar reads `~/.config/waybar/` automatically. No sway orchestrator changes needed.

---

## Scripts

All scripts live in `~/.local/share/solverforge/bin/` with the `solverforge-` prefix.

| Script | Description |
|---|---|
| `solverforge-menu` | Hierarchical system menu (wofi dmenu) |
| `solverforge-keys` | Keybinding viewer (terminal + dmenu modes) |
| `solverforge-icon-picker` | Browse 10,764 Nerd Font icons via fzf |
| `solverforge-launch-webapp` | Launch URL in chromeless browser window |
| `solverforge-launch-or-focus` | Focus existing window or launch new one |
| `solverforge-webapp-install` | Install web app as .desktop entry with icon picker |
| `solverforge-webapp-remove` | Remove installed web app |
| `solverforge-tui-install` | Install TUI app with icon picker |
| `solverforge-tui-remove` | Remove user-installed TUI app |
| `solverforge-pkg-install` | Interactive zypper package installer (fzf) |
| `solverforge-pkg-remove` | Interactive zypper package remover (fzf) |
| `solverforge-disk-install` | Install local executable as .desktop entry with icon picker |
| `solverforge-disk-remove` | Remove disk-installed app |
| `solverforge-lazyvim-install` | Bootstrap or reinitialize LazyVim with SolverForge defaults |
| `solverforge-autotile` | Autotiling daemon — dwindle/spiral layout for sway |
| `solverforge-calendar` | Calendar application |
| `solverforge-mail` | Email client |
| `solverforge-showkeys` | Toggle on-screen keystroke display (wshowkeys) |
| `solverforge-iphone-cam` | iPhone camera → virtual webcam via GStreamer (toggle) |
| `solverforge-theme-apply` | Generate theme configs from colors.toml and deploy |
| `solverforge-podman-overview` | Podman container overview with decorated headers |
| `solverforge-virsh-overview` | Virsh/libvirt VM overview with decorated headers |
| `solverforge-backup` | Restic encrypted backup |
| `solverforge-backup-prune` | Restic backup prune |
| `solverforge-waybar-podman` | Waybar module: Podman container count |
| `solverforge-waybar-updates` | Waybar module: zypper pending updates |
| `solverforge-waybar-cava` | Waybar module: cava audio visualizer (braille + rainbow) |
| `solverforge-waybar-ollama` | Waybar module: Ollama loaded model count |
| `solverforge-waybar-virsh` | Waybar module: libvirt VM status |
| `solverforge-waybar-notifications` | Waybar module: SwayNC notification indicator |
| `solverforge-outlook-send` | OAuth2 email sender (Python) |
| `solverforge-himalaya` | Himalaya email client (symlink) |
| `zen` | Zen browser (symlink to /opt/zen/zen) |

---

## Theme System

SolverForge Linux uses a centralized theme system where all colors are defined once and propagated to every application.

### Structure

```
default/theme/
    colors.toml              Single source of truth for all colors
    templates/               Mustache-style templates (*.tpl)
    generated/               Materialized configs (git-tracked output)
    overrides/               App-specific overrides that bypass templating
```

### Templates

Templates in `default/theme/templates/` cover 19 applications:

| Template | Target |
|---|---|
| `kitty.conf.tpl` | Kitty terminal colors |
| `sway-colors.conf.tpl` | Sway window border colors |
| `fzf.env.tpl` | fzf color scheme (env vars) |
| `lazygit.yml.tpl` | Lazygit theme |
| `delta.gitconfig.tpl` | Git delta syntax highlighter |
| `eza.env.tpl` | eza file listing colors |
| `swaylock.conf.tpl` | Swaylock screen locker |
| `swaync.css.tpl` | SwayNC notification center |
| `gtk-colors.css.tpl` | GTK shared color definitions |
| `gtk3-settings.ini.tpl` | GTK3 settings |
| `gtk4-settings.ini.tpl` | GTK4 settings |
| `qt5ct-hackerman.conf.tpl` | Qt5 color scheme |
| `kdeglobals.tpl` | KDE/Qt global colors |
| `kdedefaults-kdeglobals.tpl` | KDE defaults |
| `hackerman.colors.tpl` | KDE color scheme file |
| `yazi-theme.toml.tpl` | Yazi file manager theme |
| `mc-hackerman.ini.tpl` | Midnight Commander skin |
| `tmux.conf.tpl` | Tmux status bar colors |

### Overrides

`default/theme/overrides/btop.theme` — btop uses its own theme format that doesn't fit the template system, so its theme file is maintained directly.

### Usage

```bash
solverforge-theme-apply
```

Reads `colors.toml`, processes all templates into `generated/`, deploys overrides, wires symlinks, and reloads affected applications.

---

## LazyVim Integration

SolverForge Linux ships default LazyVim configuration in `default/nvim/`:

- **Theme:** Hackerman (aether.nvim) with transparent backgrounds
- **File explorer:** Right-side via Snacks.nvim picker
- **Colorschemes:** Evangelion + Nightfox available
- **Language extras:** C/C++, CMake, Docker, Git, Go, Helm, Java, JSON, Python, Ruby, Rust, SQL, Tailwind, Terraform, TOML, YAML
- **Formatting:** StyLua 2-space indent, 120 columns

Plugin specs are symlinked from `~/.config/nvim/lua/plugins/` into the default layer. Run `solverforge-lazyvim-install` to bootstrap a fresh install or reset to defaults.

---

## TUI App System

Any terminal application can be added to the menu as a floating window:

```
solverforge-tui-install
```

This prompts for a name, command, interactive flag, and an icon (via the full Nerd Font picker). The config is saved to `~/.config/solverforge/tui-apps/<slug>.conf` and immediately appears in the TUI Apps submenu. All TUI apps launch as floating windows at the size defined in `rules.conf` (single source of truth).

---

## Web App System

Web apps are installed as `.desktop` entries that launch in a chromeless browser window:

```
solverforge-webapp-install
```

This prompts for a name, URL, and icon. The resulting `.desktop` file calls `solverforge-launch-webapp`, which uses `$BROWSER` (default: `zen`).

---

## Menu Extensions

Add custom menu entries by editing `~/.config/solverforge/extensions/menu.sh`. This file is sourced by `solverforge-menu` and has access to all its helpers:

- `menu "Prompt" "entries"` -- wofi dmenu with SolverForge styling
- `float_term "command"` -- run in floating terminal, wait for keypress
- `float_interactive "command"` -- run in floating terminal (stays open)
- `open_in_editor "file"` -- open in `$EDITOR` in floating terminal

Use `solverforge-icon-picker` to browse the full Nerd Font icon set when choosing icons for custom entries.

---

## Runtime Variables

| Variable | Default | Used by |
|---|---|---|
| `$BROWSER` | `zen` | `solverforge-launch-webapp`, sway `$browser` |
| `$EDITOR` | `nvim` | `open_in_editor()` in menu |
| `$SWAYLOCK_CONFIG` | `/etc/swaylock/openSUSEway.conf` | System menu Lock |
| `$SOLVERFORGE_PATH` | `~/.local/share/solverforge` | All scripts |
| `$XDG_RUNTIME_DIR` | system default | Materialized CSS cache |

---

## Dependencies

**Required:** sway, wofi, kitty, waybar, swaylock, swayidle, wtype, grim, slurp, wl-clipboard, libnotify-tools, fzf, jq, git, curl, bat, eza, delta, qt5ct, qt6ct, FiraCode Nerd Font, Symbols Nerd Font

**Optional:** zen, chromium, neovim, fastfetch, lolcat, btop, cava, lazygit, nmtui, bluetoothctl, pavucontrol, restic, ImageMagick, emacs, solaar, wayscriber, voxtype, podman, python3, ollama, libvirt/virsh, tmux, yazi, mc (midnight commander)

```bash
# Required
sudo zypper install sway wofi kitty waybar swaylock swayidle wtype grim slurp \
  wl-clipboard libnotify-tools fzf jq git curl bat eza git-delta qt5ct qt6ct

# Optional
sudo zypper install neovim fastfetch btop lazygit NetworkManager \
  bluez-tools pavucontrol restic ImageMagick emacs solaar python3 \
  ollama libvirt-client tmux yazi mc

# Zen browser (manual install — not in zypper repos)
# https://zen-browser.app — extract to /opt/zen/
# SolverForge symlinks /opt/zen/zen into its bin/ directory automatically.
```

See `default/dependencies.txt` for the full manifest with per-script attribution.

---

## File Map

```
~/.local/share/solverforge/
    version
    README.md
    bin/
        solverforge-autotile
        solverforge-backup
        solverforge-backup-prune
        solverforge-calendar
        solverforge-disk-install
        solverforge-disk-remove
        solverforge-himalaya              → /opt/himalaya/target/release/himalaya
        solverforge-icon-picker
        solverforge-iphone-cam
        solverforge-keys
        solverforge-launch-or-focus
        solverforge-launch-webapp
        solverforge-lazyvim-install
        solverforge-mail
        solverforge-menu
        solverforge-outlook-send
        solverforge-pkg-install
        solverforge-pkg-remove
        solverforge-podman-overview
        solverforge-showkeys
        solverforge-theme-apply
        solverforge-tui-install
        solverforge-tui-remove
        solverforge-virsh-overview
        solverforge-waybar-cava
        solverforge-waybar-notifications
        solverforge-waybar-ollama
        solverforge-waybar-podman
        solverforge-waybar-updates
        solverforge-waybar-virsh
        solverforge-webapp-install
        solverforge-webapp-remove
        zen                               → /opt/zen/zen
    default/
        sway/
            bindings.conf
            rules.conf
            output.conf
            input.conf
            autostart.conf
            voxtype.conf
        wofi/
            menu-style.css
            banner.svg
        kitty/
            mappings.conf
        bash/
            rc
            keybindings
            backup.conf
        zsh/
            solverforge-aliases.zsh
            solverforge-env.zsh
            solverforge-envs.zsh
            solverforge-functions.zsh
            solverforge-hm.zsh
            solverforge-init.zsh
            solverforge-keybindings.zsh
            solverforge-prompt.zsh
            solverforge-restic.zsh
            solverforge-shell.zsh
        nvim/
            lazyvim.json
            stylua.toml
            plugins/
                solverforge-theme.lua
                solverforge-explorer.lua
                solverforge-colorschemes.lua
                solverforge-extras.lua
        waybar/
            config
            style.css
            solverforge-icon.svg
        theme/
            colors.toml
            templates/
                kitty.conf.tpl
                sway-colors.conf.tpl
                fzf.env.tpl
                lazygit.yml.tpl
                delta.gitconfig.tpl
                eza.env.tpl
                swaylock.conf.tpl
                swaync.css.tpl
                gtk-colors.css.tpl
                gtk3-settings.ini.tpl
                gtk4-settings.ini.tpl
                qt5ct-hackerman.conf.tpl
                kdeglobals.tpl
                kdedefaults-kdeglobals.tpl
                hackerman.colors.tpl
                yazi-theme.toml.tpl
                mc-hackerman.ini.tpl
                tmux.conf.tpl
            generated/
                (materialized configs — one per template + btop.theme)
            overrides/
                btop.theme
        nerd-glyphs.json
        nerd-icons.txt
        dependencies.txt

~/.config/waybar/
    config                        Symlink to default layer
    style.css                     Symlink to default layer

~/.config/solverforge/
    backup.conf
    extensions/
        menu.sh
    tui-apps/

~/.config/sway/config.d/
    solverforge.conf              Orchestrator

~/.config/environment.d/
    solverforge.conf              PATH for systemd user sessions
```
