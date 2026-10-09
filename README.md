# Animated Cursors

Parametric Xcursor themes designed for reversible cursor transitions in a
patched Niri. Every cursor is drawn from code, so transitions interpolate the
design itself rather than cross-fading images. Unpatched compositors and X11
applications see ordinary (animated) Xcursor themes.

Each theme section below previews every cursor and transition at 64 px
nominal size. Idle loops play in real time after their rest; transitions are
slowed and play both ways with pauses at the ends (the real ones last 120 ms).

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

Each cursor rests for 1 to 2 seconds, then plays a short springy idle gesture:
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


#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/shapes/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/shapes/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/shapes/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | <img src="previews/shapes/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/shapes/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/shapes/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/shapes/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/shapes/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/shapes/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> | <img src="previews/shapes/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> |
| <img src="previews/shapes/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/shapes/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/shapes/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/shapes/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/shapes/text.gif" alt="text" width="96"><br><sub>text</sub> |
| <img src="previews/shapes/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/shapes/wait.gif" alt="wait" width="96"><br><sub>wait</sub> | <img src="previews/shapes/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/shapes/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/shapes/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/shapes/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/shapes/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/shapes/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> | <img src="previews/shapes/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> |
| <img src="previews/shapes/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/shapes/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/shapes/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/shapes/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/shapes/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/shapes/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/shapes/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/shapes/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/shapes/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> | <img src="previews/shapes/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> |
| <img src="previews/shapes/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/shapes/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/shapes/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/shapes/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> | |

### Dot (`animated_dot_cursors`)

Minimal dots and rings built from keyed bars, chevrons and arcs. Shared parts
stretch, slide and rotate between shapes; parts present on one side grow from
an anchor, ripple in, or spiral out. A transition played backwards is exactly
the opposite transition, which the tests check for every pair.

Idle loops rest, then play a small gesture: the dot beats twice, the link ring
ripples out, carets breathe, resize chevrons nudge, fingers lift off the palm
one by one, the fist squeezes, badges spin or shake, the not-allowed slash
flips end over end, crosshair ticks pulse inward, and the zoom glyph turns.


#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/dot/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/dot/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/dot/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | <img src="previews/dot/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/dot/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/dot/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/dot/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/dot/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/dot/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> | <img src="previews/dot/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> |
| <img src="previews/dot/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/dot/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/dot/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/dot/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/dot/text.gif" alt="text" width="96"><br><sub>text</sub> |
| <img src="previews/dot/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/dot/wait.gif" alt="wait" width="96"><br><sub>wait</sub> | <img src="previews/dot/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/dot/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/dot/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/dot/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/dot/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/dot/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> | <img src="previews/dot/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> |
| <img src="previews/dot/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/dot/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/dot/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/dot/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/dot/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/dot/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/dot/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/dot/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/dot/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> | <img src="previews/dot/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> |
| <img src="previews/dot/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/dot/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/dot/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/dot/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> | |

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
Cursors are drawn 1.5 times their nominal size on canvases enlarged to match
(36 px at size 24), so they read like conventional themes. Transition frames
use a canvas twice that, centered on the hotspot.
Only applications using the cursor-shape protocol get transitions.
