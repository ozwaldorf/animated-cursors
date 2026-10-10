//! Cursor shapes shared by every theme, with their names and the transition pairs.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    Default,
    Pointer,
    Text,
    VerticalText,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    Grab,
    Grabbing,
    Copy,
    NoDrop,
    NotAllowed,
    Progress,
    Wait,
    Crosshair,
    ZoomIn,
    ZoomOut,
}

impl Shape {
    pub const ALL: [Shape; 19] = [
        Shape::Default,
        Shape::Pointer,
        Shape::Text,
        Shape::VerticalText,
        Shape::EwResize,
        Shape::NsResize,
        Shape::NeswResize,
        Shape::NwseResize,
        Shape::ColResize,
        Shape::Grab,
        Shape::Grabbing,
        Shape::Copy,
        Shape::NoDrop,
        Shape::NotAllowed,
        Shape::Progress,
        Shape::Wait,
        Shape::Crosshair,
        Shape::ZoomIn,
        Shape::ZoomOut,
    ];

    /// Wayland cursor-shape name.
    pub fn name(self) -> &'static str {
        match self {
            Shape::Default => "default",
            Shape::Pointer => "pointer",
            Shape::Text => "text",
            Shape::VerticalText => "vertical-text",
            Shape::EwResize => "ew-resize",
            Shape::NsResize => "ns-resize",
            Shape::NeswResize => "nesw-resize",
            Shape::NwseResize => "nwse-resize",
            Shape::ColResize => "col-resize",
            Shape::Grab => "grab",
            Shape::Grabbing => "grabbing",
            Shape::Copy => "copy",
            Shape::NoDrop => "no-drop",
            Shape::NotAllowed => "not-allowed",
            Shape::Progress => "progress",
            Shape::Wait => "wait",
            Shape::Crosshair => "crosshair",
            Shape::ZoomIn => "zoom-in",
            Shape::ZoomOut => "zoom-out",
        }
    }

    /// Other Wayland cursor-shape names drawn with this shape.
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Shape::Default => &["context-menu", "help"],
            Shape::EwResize => &["e-resize", "w-resize"],
            Shape::NsResize => &["n-resize", "s-resize", "row-resize"],
            Shape::NeswResize => &["ne-resize", "sw-resize"],
            Shape::NwseResize => &["nw-resize", "se-resize"],
            Shape::Grabbing => &["move", "all-resize", "all-scroll"],
            Shape::Copy => &["alias"],
            Shape::Crosshair => &["cell"],
            _ => &[],
        }
    }

    pub fn from_name(name: &str) -> Option<Shape> {
        Shape::ALL.into_iter().find(|shape| shape.name() == name || shape.aliases().contains(&name))
    }

    /// Busy cursors, which loop while an application works.
    pub fn is_busy(self) -> bool {
        matches!(self, Shape::Progress | Shape::Wait)
    }
}

pub const PAIRS: [(Shape, Shape); 24] = [
    (Shape::Default, Shape::Pointer),
    (Shape::Default, Shape::Text),
    (Shape::Pointer, Shape::Text),
    (Shape::Default, Shape::EwResize),
    (Shape::Default, Shape::NsResize),
    (Shape::Default, Shape::NeswResize),
    (Shape::Default, Shape::NwseResize),
    (Shape::Default, Shape::ColResize),
    (Shape::Default, Shape::Grab),
    (Shape::Grab, Shape::Grabbing),
    (Shape::Default, Shape::Grabbing),
    (Shape::Default, Shape::Progress),
    (Shape::Default, Shape::Wait),
    (Shape::Grabbing, Shape::Copy),
    (Shape::Grabbing, Shape::NotAllowed),
    (Shape::Grabbing, Shape::NoDrop),
    (Shape::Default, Shape::Crosshair),
    (Shape::Text, Shape::VerticalText),
    (Shape::Text, Shape::EwResize),
    (Shape::Text, Shape::NsResize),
    (Shape::Text, Shape::NeswResize),
    (Shape::Text, Shape::NwseResize),
    (Shape::Text, Shape::ColResize),
    (Shape::ZoomIn, Shape::ZoomOut),
];

/// Legacy X11, Qt and toolkit names.
pub const LEGACY_ALIASES: [(Shape, &[&str]); 16] = [
    (
        Shape::Default,
        &[
            "arrow",
            "left_ptr",
            "top_left_arrow",
            "right_ptr",
            "center_ptr",
            "left_ptr_help",
            "question_arrow",
            "whats_this",
            "dnd-ask",
            "up-arrow",
            "down-arrow",
            "left-arrow",
            "right-arrow",
            "sb_up_arrow",
            "sb_down_arrow",
            "sb_left_arrow",
            "sb_right_arrow",
            "left_tee",
            "right_tee",
            "top_tee",
            "bottom_tee",
            "wayland-cursor",
            "5c6cd98b3f3ebcb1f9c7f1c204630408",
            "d9ce0ab605698f320427677b458ad60b",
        ],
    ),
    (
        Shape::Pointer,
        &[
            "hand",
            "hand1",
            "hand2",
            "pointing_hand",
            "9d800788f1b08800ae810202380a0822",
            "e29285e634086352946a0e7090d73106",
        ],
    ),
    (Shape::Text, &["xterm", "ibeam"]),
    (
        Shape::EwResize,
        &[
            "h_double_arrow",
            "sb_h_double_arrow",
            "size_hor",
            "size-hor",
            "left_side",
            "right_side",
            "028006030e0e7ebffc7f7070c0600140",
            "14fef782d02440884392942c11205230",
        ],
    ),
    (
        Shape::NsResize,
        &[
            "v_double_arrow",
            "sb_v_double_arrow",
            "double_arrow",
            "size_ver",
            "size-ver",
            "top_side",
            "bottom_side",
            "split_v",
            "00008160000006810000408080010102",
            "2870a09082c103050810ffdffffe0204",
        ],
    ),
    (
        Shape::NeswResize,
        &[
            "bd_double_arrow",
            "size_bdiag",
            "size-bdiag",
            "top_right_corner",
            "bottom_left_corner",
            "ur_angle",
            "ll_angle",
            "c7088f0f3e6c8088236ef8e1e3e70000",
        ],
    ),
    (
        Shape::NwseResize,
        &[
            "fd_double_arrow",
            "size_fdiag",
            "size-fdiag",
            "top_left_corner",
            "bottom_right_corner",
            "ul_angle",
            "lr_angle",
            "fcf1c3c7cd4491d801f1e1c78f100000",
        ],
    ),
    (Shape::ColResize, &["split_h"]),
    (Shape::Grab, &["openhand"]),
    (
        Shape::Grabbing,
        &[
            "closedhand",
            "dnd-move",
            "dnd-none",
            "fleur",
            "size_all",
            "4498f0e0c1937ffe01fd06f973665830",
            "9081237383d90e509aa00f00170e968f",
            "fcf21c00b30f7e3f83fe0dfd12e71cff",
        ],
    ),
    (
        Shape::Progress,
        &[
            "left_ptr_watch",
            "half-busy",
            "00000000000000020006000e7e9ffc3f",
            "08e8e1c95fe2fc01f976f1e063a24ccd",
            "3ecb610c1bf2410f44200f48c40d3599",
        ],
    ),
    (Shape::Wait, &["watch"]),
    (
        Shape::Copy,
        &[
            "dnd-copy",
            "link",
            "dnd-link",
            "1081e37283d90000800003c07f3ef6bf",
            "6407b0e94181790501fd1e167b474872",
            "b66166c04f8c3109214a4fbd64a50fc8",
            "3085a0e285430894940527032f8b26df",
            "640fb0e74195791501fd1ed57b41487f",
            "a2a266d0498c3104214a47bd64ab0fc8",
        ],
    ),
    (
        Shape::NotAllowed,
        &["circle", "crossed_circle", "pirate", "X_cursor", "x-cursor", "03b6e0fcb3499374a867c041f52298f0"],
    ),
    (Shape::NoDrop, &["dnd-no-drop", "forbidden"]),
    (
        Shape::Crosshair,
        &[
            "cross",
            "cross_reverse",
            "diamond_cross",
            "tcross",
            "plus",
            "color-picker",
            "target",
            "dotbox",
            "dot_box_mask",
            "draped_box",
            "icon",
            "pencil",
            "draft",
            "draft_large",
            "draft_small",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_resolve_to_shapes() {
        for shape in Shape::ALL {
            assert_eq!(Shape::from_name(shape.name()), Some(shape));
        }
        assert_eq!(Shape::from_name("row-resize"), Some(Shape::NsResize));
        assert_eq!(Shape::from_name("all-scroll"), Some(Shape::Grabbing));
        assert_eq!(Shape::from_name("nonexistent"), None);
    }
}
