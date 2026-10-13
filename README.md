# Animated Cursors

Xcursor themes drawn from code, morphing between cursors in compositors that
support it and plain animated cursors everywhere else.

- [Compositor support](#compositor-support)
- [Palettes](#palettes)
- [Themes](#themes)
  - [Shapes](#shapes-animated_shapes_cursors)
  - [Dot](#dot-animated_dot_cursors)
  - [Fluid](#fluid-animated_fluid_cursors)
  - [Hand](#hand-animated_hand_cursors)
- [Build](#build)
- [License](#license)

## Compositor support

Transitions need a compositor that plays them. See [`patches/`](patches/README.md)
for the spec and a Niri patch.

## Palettes

| | Carburetor | Catppuccin Mocha | Gruvbox dark |
| --- | --- | --- | --- |
| Shapes | <img src="previews/palettes/shapes-carburetor.gif" alt="shapes in carburetor" width="240"> | <img src="previews/palettes/shapes-catppuccin.gif" alt="shapes in catppuccin" width="240"> | <img src="previews/palettes/shapes-gruvbox.gif" alt="shapes in gruvbox" width="240"> |
| Dot | <img src="previews/palettes/dot-carburetor.gif" alt="dot in carburetor" width="240"> | <img src="previews/palettes/dot-catppuccin.gif" alt="dot in catppuccin" width="240"> | <img src="previews/palettes/dot-gruvbox.gif" alt="dot in gruvbox" width="240"> |
| Fluid | <img src="previews/palettes/fluid-carburetor.gif" alt="fluid in carburetor" width="240"> | <img src="previews/palettes/fluid-catppuccin.gif" alt="fluid in catppuccin" width="240"> | <img src="previews/palettes/fluid-gruvbox.gif" alt="fluid in gruvbox" width="240"> |
| Hand | <img src="previews/palettes/hand-carburetor.gif" alt="hand in carburetor" width="240"> | <img src="previews/palettes/hand-catppuccin.gif" alt="hand in catppuccin" width="240"> | <img src="previews/palettes/hand-gruvbox.gif" alt="hand in gruvbox" width="240"> |

Pick one with `--palette carburetor|catppuccin|gruvbox` or a path to your own
`role = #rrggbb` file (see `crates/<theme>/colors`). With the flake:

```nix
animated-cursors.packages.${system}.fluid.override {
  palette = "catppuccin";
  colors.arrow = "#f5c2e7";
}
```

## Themes

Previews are 64 px; transitions are slowed down and play both ways.

### Shapes (`animated_shapes_cursors`)

Material 3 Expressive shapes.

#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/shapes/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/shapes/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/shapes/text.gif" alt="text" width="96"><br><sub>text</sub> | <img src="previews/shapes/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/shapes/wait.gif" alt="wait" width="96"><br><sub>wait</sub> |
| <img src="previews/shapes/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/shapes/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/shapes/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/shapes/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/shapes/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> |
| <img src="previews/shapes/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> | <img src="previews/shapes/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/shapes/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | <img src="previews/shapes/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/shapes/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/shapes/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/shapes/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/shapes/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/shapes/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/shapes/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/shapes/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/shapes/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/shapes/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> | <img src="previews/shapes/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> |
| <img src="previews/shapes/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/shapes/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/shapes/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/shapes/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/shapes/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/shapes/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/shapes/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/shapes/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/shapes/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/shapes/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> |
| <img src="previews/shapes/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> | <img src="previews/shapes/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/shapes/text-to-col-resize.gif" alt="text / col-resize" width="96"><br><sub>text / col-resize</sub> | <img src="previews/shapes/text-to-ew-resize.gif" alt="text / ew-resize" width="96"><br><sub>text / ew-resize</sub> | <img src="previews/shapes/text-to-nesw-resize.gif" alt="text / nesw-resize" width="96"><br><sub>text / nesw-resize</sub> |
| <img src="previews/shapes/text-to-ns-resize.gif" alt="text / ns-resize" width="96"><br><sub>text / ns-resize</sub> | <img src="previews/shapes/text-to-nwse-resize.gif" alt="text / nwse-resize" width="96"><br><sub>text / nwse-resize</sub> | <img src="previews/shapes/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/shapes/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> |  |

### Dot (`animated_dot_cursors`)

Minimal dots and rings.

#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/dot/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/dot/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/dot/text.gif" alt="text" width="96"><br><sub>text</sub> | <img src="previews/dot/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/dot/wait.gif" alt="wait" width="96"><br><sub>wait</sub> |
| <img src="previews/dot/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/dot/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/dot/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/dot/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/dot/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> |
| <img src="previews/dot/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> | <img src="previews/dot/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/dot/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | <img src="previews/dot/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/dot/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/dot/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/dot/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/dot/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/dot/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/dot/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/dot/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/dot/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/dot/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> | <img src="previews/dot/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> |
| <img src="previews/dot/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/dot/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/dot/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/dot/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/dot/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/dot/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/dot/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/dot/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/dot/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/dot/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> |
| <img src="previews/dot/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> | <img src="previews/dot/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/dot/text-to-col-resize.gif" alt="text / col-resize" width="96"><br><sub>text / col-resize</sub> | <img src="previews/dot/text-to-ew-resize.gif" alt="text / ew-resize" width="96"><br><sub>text / ew-resize</sub> | <img src="previews/dot/text-to-nesw-resize.gif" alt="text / nesw-resize" width="96"><br><sub>text / nesw-resize</sub> |
| <img src="previews/dot/text-to-ns-resize.gif" alt="text / ns-resize" width="96"><br><sub>text / ns-resize</sub> | <img src="previews/dot/text-to-nwse-resize.gif" alt="text / nwse-resize" width="96"><br><sub>text / nwse-resize</sub> | <img src="previews/dot/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/dot/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> |  |

### Fluid (`animated_fluid_cursors`)

Liquid metaballs.

#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/fluid/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/fluid/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/fluid/text.gif" alt="text" width="96"><br><sub>text</sub> | <img src="previews/fluid/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/fluid/wait.gif" alt="wait" width="96"><br><sub>wait</sub> |
| <img src="previews/fluid/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/fluid/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/fluid/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/fluid/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/fluid/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> |
| <img src="previews/fluid/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> | <img src="previews/fluid/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/fluid/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | <img src="previews/fluid/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/fluid/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/fluid/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/fluid/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/fluid/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/fluid/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/fluid/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/fluid/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/fluid/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/fluid/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> | <img src="previews/fluid/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> |
| <img src="previews/fluid/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/fluid/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/fluid/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/fluid/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/fluid/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/fluid/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/fluid/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/fluid/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/fluid/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/fluid/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> |
| <img src="previews/fluid/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> | <img src="previews/fluid/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/fluid/text-to-col-resize.gif" alt="text / col-resize" width="96"><br><sub>text / col-resize</sub> | <img src="previews/fluid/text-to-ew-resize.gif" alt="text / ew-resize" width="96"><br><sub>text / ew-resize</sub> | <img src="previews/fluid/text-to-nesw-resize.gif" alt="text / nesw-resize" width="96"><br><sub>text / nesw-resize</sub> |
| <img src="previews/fluid/text-to-ns-resize.gif" alt="text / ns-resize" width="96"><br><sub>text / ns-resize</sub> | <img src="previews/fluid/text-to-nwse-resize.gif" alt="text / nwse-resize" width="96"><br><sub>text / nwse-resize</sub> | <img src="previews/fluid/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/fluid/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> |  |

### Hand (`animated_hand_cursors`)

Cel-shaded 3D gloves after Master Hand.

#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/hand/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/hand/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/hand/text.gif" alt="text" width="96"><br><sub>text</sub> | <img src="previews/hand/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/hand/wait.gif" alt="wait" width="96"><br><sub>wait</sub> |
| <img src="previews/hand/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/hand/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/hand/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/hand/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/hand/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> |
| <img src="previews/hand/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> | <img src="previews/hand/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/hand/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | <img src="previews/hand/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/hand/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/hand/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/hand/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/hand/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/hand/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/hand/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/hand/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/hand/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/hand/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> | <img src="previews/hand/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> |
| <img src="previews/hand/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/hand/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/hand/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/hand/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/hand/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/hand/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/hand/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/hand/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/hand/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/hand/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> |
| <img src="previews/hand/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> | <img src="previews/hand/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/hand/text-to-col-resize.gif" alt="text / col-resize" width="96"><br><sub>text / col-resize</sub> | <img src="previews/hand/text-to-ew-resize.gif" alt="text / ew-resize" width="96"><br><sub>text / ew-resize</sub> | <img src="previews/hand/text-to-nesw-resize.gif" alt="text / nesw-resize" width="96"><br><sub>text / nesw-resize</sub> |
| <img src="previews/hand/text-to-ns-resize.gif" alt="text / ns-resize" width="96"><br><sub>text / ns-resize</sub> | <img src="previews/hand/text-to-nwse-resize.gif" alt="text / nwse-resize" width="96"><br><sub>text / nwse-resize</sub> | <img src="previews/hand/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/hand/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> |  |

## Build

```sh
nix build .#shapes                       # theme in result/share/icons
nix build .#dot
nix build .#fluid
nix build .#hand
nix build .#fluid-catppuccin             # every theme has -catppuccin and -gruvbox
nix build .#niri --out-link result-niri  # patched Niri 26.04
```

Locally:

```sh
nix develop
cargo test
cargo run --release -p fluid-cursors -- --output build/fluid --preview previews/fluid
cargo run --release -p fluid-cursors -- --output build/fluid --sizes 24,48 --palette gruvbox
cargo run --release -p fluid-cursors -- --sheet previews/palettes/fluid-gruvbox.gif --palette gruvbox
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
