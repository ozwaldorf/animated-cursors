# Animated Cursors

Parametric Xcursor themes designed for reversible cursor transitions in a
patched Niri. Every cursor is drawn from code, so transitions interpolate the
design itself rather than cross-fading images. Unpatched compositors and X11
applications see ordinary (animated) Xcursor themes.

Open `previews/index.html` for every cursor and transition in both themes.

## Themes

### Shapes (`animated_shapes_cursors`)

Material 3 Expressive inspired. Every cursor is a shape from the M3 shape
library (rounded polygons, cookies, clovers, sunny and soft-burst stars,
flowers, pills) in its own vivid color, with white glyphs:

| Cursor | Shape | Color |
| --- | --- | --- |
| Arrow | Rounded triangle, tip on the hotspot | Violet |
| Link | Four-leaf clover with a center dot | Coral |
| Text, vertical text | Pill | Indigo |
| Resize | Pill with chevrons; column resize is a rounded square | Teal |
| Grab, grabbing | Open 8-petal bloom that closes into a tight cookie | Sky, navy |
| Copy | Flower with a plus | Green |
| Not allowed, no drop | Octagon with a bar; closed bloom with a ban sign | Red |
| Progress, wait | Loading indicator cycling square, triangle, hexagon, pill, pentagon, diamond and circle, each in its own color | Multi |
| Crosshair | Four triangles around a dot | Charcoal |
| Zoom | Sunny and 12-sided cookie with plus and minus | Magenta |

Each cursor rests for a few seconds, then plays a short springy idle gesture:
the arrow nods about its tip, the clover, bloom, flower and zoom shapes twirl
by one of their symmetries (so the loop closes seamlessly), resize chevrons
nudge outward, grabbing squeezes, no-drop and not-allowed shake, and the
crosshair pulses. Busy cursors loop the loading indicator instead, one shape
every 0.7 seconds.

Every cursor clicks at the arrow tip, and every shape sits centered on the
arrow body, so morphs happen in place. For centered shapes such as the
clover, pill or octagon, the click point is at their upper left.

Shapes are polar outlines (radii at evenly spaced angles), so any two morph
by interpolating radii. Transitions spring with a slight overshoot and twist,
sweep color through OKLab, morph glyphs that share a key, and pop the rest in
or out.

### Dot (`animated_dot_cursors`)

Minimal dots and rings built from keyed bars, chevrons and arcs. Shared parts
stretch, slide and rotate between shapes; parts present on one side grow from
an anchor, ripple in, or spiral out. A transition played backwards is exactly
the opposite transition, which the tests check for every pair.

Idle loops rest, then play a small gesture: the dot beats twice, the link ring
ripples out, carets breathe, resize chevrons nudge, fingers lift off the palm
one by one, the fist squeezes, badges spin or shake, the not-allowed slash
flips end over end, crosshair ticks pulse inward, and the zoom glyph turns.

## Layout

- `crates/core`: shape names and aliases, motion and paint helpers, the theme
  writer (Xcursor files, transition files, symlinks, GIF previews), the CLI,
  and the consistency checks every theme runs.
- `crates/dot`, `crates/shapes`: one crate per theme. Each implements
  `cursor_core::Theme`: hotspots, cursor frames, transition frames and drawing.

A new theme is a crate with a `Theme` implementation, a three-line `main.rs`
calling `cursor_core::cli::run`, and a test calling `cursor_core::check::theme`.

The checks require exact hotspots at every size, transition endpoints identical
to the ordinary cursors, nothing clipped at the canvas edge, and working aliases.

## Build

```sh
nix build .#shapes                       # theme in result/share/icons
nix build .#dot
nix build .#niri --out-link result-niri  # patched Niri 26.04
```

Locally:

```sh
nix develop
cargo test
cargo run --release -p shapes-cursors -- --output build/shapes --preview previews/shapes
cargo run --release -p dot-cursors -- --output build/dot --preview previews/dot
```

## Niri

Apply `patches/niri-cursor-transitions.patch` to Niri 26.04, then select a
theme with Home Manager's `home.pointerCursor` and in Niri:

```kdl
cursor {
    xcursor-theme "animated_shapes_cursors"
    xcursor-size 24
}
```

A transition is an ordinary animated Xcursor file named
`cursors/<from>-to-<to>` with Wayland cursor-shape names, 24 frames at 5 ms.
One file serves both directions, and shape aliases share it through symlinks.
Transition frames use a canvas twice the nominal size centered on the hotspot.
Only applications using the cursor-shape protocol get transitions.
