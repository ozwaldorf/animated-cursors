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

Idle loops rest, then play a small gesture: the dot beats twice, the fingertip
pointer taps, carets breathe, resize chevrons nudge, fingers lift off the palm
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

### Fluid (`animated_fluid_cursors`)

Liquid metaballs. Every cursor is a few drops (circles, capsules and round
cones) summed into one distance field through an exponential smooth union, so
nearby drops melt together with soft necks. Carved drops cut white glyphs out
of the liquid, and inlaid drops refill holes, as in the zoom lens and the ban
sign. Each cursor has its own color, and where differently colored drops meet,
their colors blend:

| Cursor | Drops | Color |
| --- | --- | --- |
| Arrow | Teardrop, tip on the hotspot | Blue |
| Link | Upright teardrop with a carved hole | Coral |
| Text, vertical text | Stem melting into two serifs | Indigo |
| Resize | Bar with chevron arrowheads | Teal |
| Grab, grabbing | Palm with a thumb and three fingers that pull in to knuckles | Sky, navy |
| Copy, no drop | Fist with a badge drop carved with a plus or a slash | Green, red |
| Not allowed | Ring with an inlaid slash | Red |
| Progress | Teardrop with two drops circling its belly | Blue, amber, coral |
| Wait | Core with three orbiting drops | Violet, magenta, amber, teal |
| Crosshair | Dot and four ticks | Charcoal |
| Zoom | Lens ring with an inlaid plus or minus, and a handle | Magenta |

Drops sharing a key flow into each other while the union gets briefly gooier
mid-transition. Drops present on one side bud out of an anchor or melt back
into it, with their border shrinking along with them. A transition played
backwards is exactly the opposite transition, which the tests check for every
pair.

Idle loops rest, then play a liquid gesture: a drip sags from the arrow and
springs back with a jiggle, the link drop taps, a bead runs down the text
stem, arrowheads pull off the resize bar, fingers stretch one after another,
the fist squeezes, the copy badge tugs away while its plus turns, the no-drop
badge shakes, the ban sign wobbles like jelly, crosshair ticks fall into the
center and pull back out, and the zoom glyph turns. Progress drops circle
continuously; wait drops are flung out and fall back in twice per turn.

#### Cursors

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/fluid/col-resize.gif" alt="col-resize" width="96"><br><sub>col-resize</sub> | <img src="previews/fluid/copy.gif" alt="copy" width="96"><br><sub>copy</sub> | <img src="previews/fluid/crosshair.gif" alt="crosshair" width="96"><br><sub>crosshair</sub> | <img src="previews/fluid/default.gif" alt="default" width="96"><br><sub>default</sub> | <img src="previews/fluid/ew-resize.gif" alt="ew-resize" width="96"><br><sub>ew-resize</sub> |
| <img src="previews/fluid/grabbing.gif" alt="grabbing" width="96"><br><sub>grabbing</sub> | <img src="previews/fluid/grab.gif" alt="grab" width="96"><br><sub>grab</sub> | <img src="previews/fluid/nesw-resize.gif" alt="nesw-resize" width="96"><br><sub>nesw-resize</sub> | <img src="previews/fluid/no-drop.gif" alt="no-drop" width="96"><br><sub>no-drop</sub> | <img src="previews/fluid/not-allowed.gif" alt="not-allowed" width="96"><br><sub>not-allowed</sub> |
| <img src="previews/fluid/ns-resize.gif" alt="ns-resize" width="96"><br><sub>ns-resize</sub> | <img src="previews/fluid/nwse-resize.gif" alt="nwse-resize" width="96"><br><sub>nwse-resize</sub> | <img src="previews/fluid/pointer.gif" alt="pointer" width="96"><br><sub>pointer</sub> | <img src="previews/fluid/progress.gif" alt="progress" width="96"><br><sub>progress</sub> | <img src="previews/fluid/text.gif" alt="text" width="96"><br><sub>text</sub> |
| <img src="previews/fluid/vertical-text.gif" alt="vertical-text" width="96"><br><sub>vertical-text</sub> | <img src="previews/fluid/wait.gif" alt="wait" width="96"><br><sub>wait</sub> | <img src="previews/fluid/zoom-in.gif" alt="zoom-in" width="96"><br><sub>zoom-in</sub> | <img src="previews/fluid/zoom-out.gif" alt="zoom-out" width="96"><br><sub>zoom-out</sub> | |

#### Transitions

| | | | | |
| --- | --- | --- | --- | --- |
| <img src="previews/fluid/default-to-col-resize.gif" alt="default / col-resize" width="96"><br><sub>default / col-resize</sub> | <img src="previews/fluid/default-to-crosshair.gif" alt="default / crosshair" width="96"><br><sub>default / crosshair</sub> | <img src="previews/fluid/default-to-ew-resize.gif" alt="default / ew-resize" width="96"><br><sub>default / ew-resize</sub> | <img src="previews/fluid/default-to-grabbing.gif" alt="default / grabbing" width="96"><br><sub>default / grabbing</sub> | <img src="previews/fluid/default-to-grab.gif" alt="default / grab" width="96"><br><sub>default / grab</sub> |
| <img src="previews/fluid/default-to-nesw-resize.gif" alt="default / nesw-resize" width="96"><br><sub>default / nesw-resize</sub> | <img src="previews/fluid/default-to-ns-resize.gif" alt="default / ns-resize" width="96"><br><sub>default / ns-resize</sub> | <img src="previews/fluid/default-to-nwse-resize.gif" alt="default / nwse-resize" width="96"><br><sub>default / nwse-resize</sub> | <img src="previews/fluid/default-to-pointer.gif" alt="default / pointer" width="96"><br><sub>default / pointer</sub> | <img src="previews/fluid/default-to-progress.gif" alt="default / progress" width="96"><br><sub>default / progress</sub> |
| <img src="previews/fluid/default-to-text.gif" alt="default / text" width="96"><br><sub>default / text</sub> | <img src="previews/fluid/default-to-wait.gif" alt="default / wait" width="96"><br><sub>default / wait</sub> | <img src="previews/fluid/grabbing-to-copy.gif" alt="grabbing / copy" width="96"><br><sub>grabbing / copy</sub> | <img src="previews/fluid/grabbing-to-no-drop.gif" alt="grabbing / no-drop" width="96"><br><sub>grabbing / no-drop</sub> | <img src="previews/fluid/grabbing-to-not-allowed.gif" alt="grabbing / not-allowed" width="96"><br><sub>grabbing / not-allowed</sub> |
| <img src="previews/fluid/grab-to-grabbing.gif" alt="grab / grabbing" width="96"><br><sub>grab / grabbing</sub> | <img src="previews/fluid/pointer-to-text.gif" alt="pointer / text" width="96"><br><sub>pointer / text</sub> | <img src="previews/fluid/text-to-vertical-text.gif" alt="text / vertical-text" width="96"><br><sub>text / vertical-text</sub> | <img src="previews/fluid/zoom-in-to-zoom-out.gif" alt="zoom-in / zoom-out" width="96"><br><sub>zoom-in / zoom-out</sub> | |

## Layout

- `crates/core`: shape names and aliases, motion, color and paint helpers,
  the theme writer (Xcursor files, transition files, symlinks, GIF previews),
  the CLI, and the consistency checks every theme runs.
- `crates/dot`, `crates/fluid`, `crates/shapes`: one crate per theme. Each
  implements `cursor_core::Theme`: hotspots, cursor frames, transition frames
  and drawing.

A new theme is a crate with a `Theme` implementation, a three-line `main.rs`
calling `cursor_core::cli::run`, and a test calling `cursor_core::check::theme`.

The checks require exact hotspots at every size, transition endpoints identical
to the ordinary cursors, nothing clipped at the canvas edge, and working aliases.

## Build

```sh
nix build .#shapes                       # theme in result/share/icons
nix build .#dot
nix build .#fluid
nix build .#niri --out-link result-niri  # patched Niri 26.04
```

Locally:

```sh
nix develop
cargo test
cargo run --release -p shapes-cursors -- --output build/shapes --preview previews/shapes
cargo run --release -p dot-cursors -- --output build/dot --preview previews/dot
cargo run --release -p fluid-cursors -- --output build/fluid --preview previews/fluid
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
