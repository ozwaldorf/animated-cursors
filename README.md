# Animated Cursors

Xcursor themes drawn from code, morphing between cursors in compositors that
support it and plain animated cursors everywhere else.

- [Compositor support](#compositor-support)
- [Themes](#themes)
  - [Shapes](#shapes-animated_shapes_cursors)
  - [Dot](#dot-animated_dot_cursors)
  - [Fluid](#fluid-animated_fluid_cursors)
  - [Hand](#hand-animated_hand_cursors)
- [Animations](#animations)
- [Palettes](#palettes)
- [Build](#build)
- [License](#license)

## Compositor support

Transitions need a compositor that plays them. See [`patches/`](patches/README.md)
for the spec and a Niri patch.

## Themes

Each preview tours a few cursors at 96 px, idling at each and morphing to the
next. Transitions are slowed down four times.

### Shapes (`animated_shapes_cursors`)

Material 3 Expressive shapes.

<img src="previews/shapes/tour.gif" alt="shapes tour" width="192">

### Dot (`animated_dot_cursors`)

Minimal dots and rings.

<img src="previews/dot/tour.gif" alt="dot tour" width="192">

### Fluid (`animated_fluid_cursors`)

Liquid metaballs.

<img src="previews/fluid/tour.gif" alt="fluid tour" width="192">

### Hand (`animated_hand_cursors`)

Cel-shaded 3D gloves after Master Hand.

<img src="previews/hand/tour.gif" alt="hand tour" width="192">

## Animations

Every transition, slowed and playing both ways, and every idle loop, at 64 px.

| | Transitions | Idle |
| --- | --- | --- |
| Shapes | <img src="previews/shapes/transitions.gif" alt="shapes transitions" width="400"> | <img src="previews/shapes/idle.gif" alt="shapes idle loops" width="400"> |
| Dot | <img src="previews/dot/transitions.gif" alt="dot transitions" width="400"> | <img src="previews/dot/idle.gif" alt="dot idle loops" width="400"> |
| Fluid | <img src="previews/fluid/transitions.gif" alt="fluid transitions" width="400"> | <img src="previews/fluid/idle.gif" alt="fluid idle loops" width="400"> |
| Hand | <img src="previews/hand/transitions.gif" alt="hand transitions" width="400"> | <img src="previews/hand/idle.gif" alt="hand idle loops" width="400"> |

## Palettes

Rows are Carburetor, Catppuccin Mocha and Gruvbox dark.

| | |
| --- | --- |
| Shapes | <img src="previews/shapes/palettes.gif" alt="shapes palettes" width="432"> |
| Dot | <img src="previews/dot/palettes.gif" alt="dot palettes" width="432"> |
| Fluid | <img src="previews/fluid/palettes.gif" alt="fluid palettes" width="432"> |
| Hand | <img src="previews/hand/palettes.gif" alt="hand palettes" width="432"> |

Pick one with `--palette carburetor|catppuccin|gruvbox` or a path to your own
`role = #rrggbb` file (see `crates/<theme>/colors`). With the flake:

```nix
animated-cursors.packages.${system}.fluid.override {
  palette = "catppuccin";
  colors.arrow = "#f5c2e7";
}
```

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
cargo run --release -p fluid-cursors -- --output build/fluid
cargo run --release -p fluid-cursors -- --output build/fluid --sizes 24,48 --palette gruvbox
cargo run --release -p fluid-cursors -- --preview previews/fluid
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
