# Compositor support

The themes are ordinary animated Xcursor themes with extra files for
transitions. Supporting transitions means teaching the compositor's cursor
manager to find those files and play them when the cursor shape changes.
Everything else keeps working as before.

## Spec

### Assets

- A transition between shapes `a` and `b` is an animated Xcursor file at
  `cursors/<a>-to-<b>` in the theme, named with cursor-shape-v1 (CSS) names.
- One file serves both directions. Look up both `<a>-to-<b>` and
  `<b>-to-<a>`. Try the lexicographically smaller name first in both
  directions so the same file always wins, and remember whether it plays
  reversed.
- Load it like any cursor: pick the image size nearest the nominal size times
  the output scale, and keep only images of that size. Its duration is the sum
  of the frame delays. Ignore files with fewer than two frames or any zero
  delay.
- Frame 0 matches `a` and the last frame matches `b`, so a transition can
  start and end without a jump. Frames carry their own hotspots, which change
  during a transition.
- No file for a pair means no transition: switch shapes immediately.

### State

Keep at most one transition: `from`, `to`, a start instant, a position in
[0, 1] at that instant, a direction, the duration, and one queued shape.

When the client requests named shape `x` and the current shape is named:

| Situation | Action |
| --- | --- |
| `x` is the current shape | Nothing. Do not restart. |
| No transition is playing | Start `current -> x` at position 0 if an asset exists, else switch. |
| Playing, `x` is `from` or `to` | Reverse in place: keep the displayed position, restart the clock, head for `x`, drop the queue. |
| Playing, `x` is anything else | Queue `x`, replacing any earlier queued shape. |

- A transition has finished once its position reaches the end it is heading
  for. If a shape is queued, start `target -> queued` from the instant it
  finished, not from when the finish was noticed, or switch directly if no
  asset exists.
- Hidden cursors, client surfaces and theme reloads cancel the transition
  immediately.

### Rendering

- Position is `p = p0 +/- elapsed / duration`, clamped to [0, 1]. Show the
  frame at `p * duration` ms, using `1 - p` for an asset that plays reversed,
  and clamp to the last frame.
- Place each frame by its own hotspot. Subtract the hotspot in logical
  floating-point coordinates before rounding to physical pixels, so it does not
  jitter at fractional scales.
- Report the cursor as animated while a transition plays, so every frame is
  redrawn.
- When it finishes, start the destination cursor's own animation from frame
  0 at the instant it finished.
- Cache textures by file name, not by shape, since one shape pair maps to a
  transition file.

## Examples

| Compositor | Version | Patch |
| --- | --- | --- |
| [Niri](https://github.com/YaLTeR/niri) | 26.04 | [`niri.patch`](niri.patch) |

### Niri

Patches `src/cursor.rs` (the `CursorManager` state machine, asset lookup and
tests) and `src/niri.rs` (rendering the chosen frame by its hotspot). The flake
builds it with its tests:

```sh
nix build .#niri
```

Then select a theme with Home Manager's `home.pointerCursor` and in Niri:

```kdl
cursor {
    xcursor-theme "animated_shapes_cursors"
    xcursor-size 48
}
```

Only applications using the cursor-shape protocol get transitions.
