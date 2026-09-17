//! Built-in terminal colour schemes: the well-known palettes as shipped by Ptyxis (see CREDITS.md),
//! which Flandre then pulls towards the wallpaper. `Flandre` itself is not here: it is derived
//! entirely from the Material scheme in `colors.rs`.

use crate::config::TermScheme;

pub struct SchemeColors {
    pub bg: &'static str,
    pub fg: &'static str,
    pub cursor: &'static str,
    pub ansi: [&'static str; 16],
}

pub struct Scheme {
    pub dark: SchemeColors,
    /// `None` = the scheme has no light variant; Flandre synthesises one from the dark colours.
    pub light: Option<SchemeColors>,
}

/// Built-in colours for every scheme except `Flandre`.
pub fn builtin(s: TermScheme) -> Option<&'static Scheme> {
    Some(match s {
        TermScheme::Flandre => return None,
        TermScheme::Gnome => &GNOME,
        TermScheme::Tango => &TANGO,
        TermScheme::Solarized => &SOLARIZED,
        TermScheme::Monokai => &MONOKAI,
        TermScheme::Gruvbox => &GRUVBOX,
        TermScheme::Dracula => &DRACULA,
        TermScheme::Nord => &NORD,
        TermScheme::Catppuccin => &CATPPUCCIN,
        TermScheme::TokyoNight => &TOKYONIGHT,
        TermScheme::Everforest => &EVERFOREST,
        TermScheme::RosePine => &ROSEPINE,
        TermScheme::Ayu => &AYU,
        TermScheme::Kanagawa => &KANAGAWA,
    })
}

static GNOME: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#1c1c1f",
        fg: "#ffffff",
        cursor: "#ffffff",
        ansi: [
            "#241f31", "#c01c28", "#2ec27e", "#f5c211", "#1e78e4", "#9841bb", "#0ab9dc", "#c0bfbc", "#5e5c64",
            "#ed333b", "#57e389", "#f8e45c", "#51a1ff", "#c061cb", "#4fd2fd", "#f6f5f4",
        ],
    },
    light: Some(SchemeColors {
        bg: "#ffffff",
        fg: "#1d1d20",
        cursor: "#1d1d20",
        ansi: [
            "#1d1d20", "#c01c28", "#26a269", "#a2734c", "#12488b", "#a347ba", "#2aa1b3", "#cfcfcf", "#5d5d5d",
            "#f66151", "#33d17a", "#e9ad0c", "#2a7bde", "#c061cb", "#33c7de", "#ffffff",
        ],
    }),
};

static TANGO: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#2e3436",
        fg: "#d3d7cf",
        cursor: "#d3d7cf",
        ansi: [
            "#2e3436", "#cc0000", "#4e9a06", "#c4a000", "#3465a4", "#75507b", "#06989a", "#d3d7cf", "#555753",
            "#ef2929", "#8ae234", "#fce94f", "#729fcf", "#ad7fa8", "#34e2e2", "#eeeeec",
        ],
    },
    light: Some(SchemeColors {
        bg: "#eeeeec",
        fg: "#2e3436",
        cursor: "#2e3436",
        ansi: [
            "#2e3436", "#cc0000", "#4e9a06", "#c4a000", "#3465a4", "#75507b", "#06989a", "#d3d7cf", "#555753",
            "#ef2929", "#8ae234", "#fce94f", "#729fcf", "#ad7fa8", "#34e2e2", "#eeeeec",
        ],
    }),
};

static SOLARIZED: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#002b36",
        fg: "#839496",
        cursor: "#839496",
        ansi: [
            "#073642", "#dc322f", "#859900", "#cf9a6b", "#268bd2", "#d33682", "#2aa198", "#eee8d5", "#657b83",
            "#d87979", "#88cf76", "#657b83", "#2699ff", "#d33682", "#43b8c3", "#fdf6e3",
        ],
    },
    light: Some(SchemeColors {
        bg: "#fdf6e3",
        fg: "#657b83",
        cursor: "#657b83",
        ansi: [
            "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5", "#002b36",
            "#cb4b16", "#586e75", "#657b83", "#839496", "#6c71c4", "#93a1a1", "#fdf6e3",
        ],
    }),
};

static MONOKAI: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#272822",
        fg: "#f8f8f2",
        cursor: "#f8f8f2",
        ansi: [
            "#75715e", "#f92672", "#a6e22e", "#f4bf75", "#66d9ef", "#ae81ff", "#2aa198", "#f9f8f5", "#272822",
            "#f92672", "#a6e22e", "#f4bf75", "#66d9ef", "#ae81ff", "#2aa198", "#f8f8f2",
        ],
    },
    light: None,
};

static GRUVBOX: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#282828",
        fg: "#ebdbb2",
        cursor: "#ebdbb2",
        ansi: [
            "#282828", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a", "#a89984", "#928374",
            "#fb4934", "#b8bb26", "#fabd2f", "#83a598", "#d3869b", "#8ec07c", "#ebdbb2",
        ],
    },
    light: Some(SchemeColors {
        bg: "#fbf1c7",
        fg: "#3c3836",
        cursor: "#3c3836",
        ansi: [
            "#fbf1c7", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a", "#7c6f64", "#928374",
            "#9d0006", "#79740e", "#b57614", "#076678", "#8f3f71", "#427b58", "#3c3836",
        ],
    }),
};

static DRACULA: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#282a36",
        fg: "#f8f8f2",
        cursor: "#f8f8f2",
        ansi: [
            "#21222c", "#ff5555", "#50fa7b", "#f1fa8c", "#bd93f9", "#ff79c6", "#8be9fd", "#f8f8f2", "#6272a4",
            "#ff6e6e", "#69ff94", "#ffffa5", "#d6acff", "#ff92df", "#a4ffff", "#ffffff",
        ],
    },
    light: Some(SchemeColors {
        bg: "#ffffff",
        fg: "#282a36",
        cursor: "#282a36",
        ansi: [
            "#f1f2ff", "#b60021", "#006800", "#515f00", "#6946a3", "#a41d74", "#006274", "#f8f8f2", "#8393c7",
            "#ac202f", "#006803", "#585e06", "#6c4993", "#962f7c", "#006465", "#595959",
        ],
    }),
};

static NORD: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#2e3440",
        fg: "#d8dee9",
        cursor: "#d8dee9",
        ansi: [
            "#3b4252", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0", "#e5e9f0", "#4c566a",
            "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#8fbcbb", "#eceff4",
        ],
    },
    light: Some(SchemeColors {
        bg: "#e5e9f0",
        fg: "#414858",
        cursor: "#414858",
        ansi: [
            "#3b4251", "#bf6069", "#a3be8b", "#eacb8a", "#81a1c1", "#b48dac", "#88c0d0", "#d8dee9", "#4c556a",
            "#bf6069", "#a3be8b", "#eacb8a", "#81a1c1", "#b48dac", "#8fbcbb", "#eceff4",
        ],
    }),
};

static CATPPUCCIN: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#1e1e2e",
        fg: "#cdd6f4",
        cursor: "#cdd6f4",
        ansi: [
            "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#bac2de", "#585b70",
            "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#a6adc8",
        ],
    },
    light: Some(SchemeColors {
        bg: "#eff1f5",
        fg: "#4c4f69",
        cursor: "#4c4f69",
        ansi: [
            "#5c5f77", "#d20f39", "#40a02b", "#df8e1d", "#1e66f5", "#ea76cb", "#179299", "#acb0be", "#6c6f85",
            "#d20f39", "#40a02b", "#df8e1d", "#1e66f5", "#ea76cb", "#179299", "#bcc0cc",
        ],
    }),
};

static TOKYONIGHT: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#1a1b26",
        fg: "#c0caf5",
        cursor: "#c0caf5",
        ansi: [
            "#414868", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#a9b1d6", "#414868",
            "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#c0caf5",
        ],
    },
    light: Some(SchemeColors {
        bg: "#e1e2e7",
        fg: "#3760bf",
        cursor: "#3760bf",
        ansi: [
            "#e9e9ed", "#f52a65", "#587539", "#8c6c3e", "#2e7de9", "#9854f1", "#007197", "#6172b0", "#a1a6c5",
            "#f52a65", "#587539", "#8c6c3e", "#2e7de9", "#9854f1", "#007197", "#3760bf",
        ],
    }),
};

static EVERFOREST: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#2d353b",
        fg: "#d3c6aa",
        cursor: "#d3c6aa",
        ansi: [
            "#4b565c", "#e67e80", "#a7c080", "#dbbc7f", "#7fbbb3", "#d699b6", "#83c092", "#d3c6aa", "#5c6a72",
            "#f85552", "#8da101", "#dfa000", "#3a94c5", "#df69ba", "#35a77c", "#dfddc8",
        ],
    },
    light: Some(SchemeColors {
        bg: "#fdf6e3",
        fg: "#5c6a72",
        cursor: "#5c6a72",
        ansi: [
            "#5c6a72", "#f85552", "#8da101", "#dfa000", "#3a94c5", "#df69ba", "#35a77c", "#dfddc8", "#4b565c",
            "#e67e80", "#a7c080", "#dbbc7f", "#7fbbb3", "#d699b6", "#83c092", "#d3c6aa",
        ],
    }),
};

static ROSEPINE: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#191724",
        fg: "#e0def4",
        cursor: "#e0def4",
        ansi: [
            "#26233a", "#eb6f92", "#9ccfd8", "#f6c177", "#31748f", "#c4a7e7", "#ebbcba", "#e0def4", "#6e6a86",
            "#eb6f92", "#9ccfd8", "#f6c177", "#31748f", "#c4a7e7", "#ebbcba", "#e0def4",
        ],
    },
    light: Some(SchemeColors {
        bg: "#faf4ed",
        fg: "#575279",
        cursor: "#575279",
        ansi: [
            "#f2e9e1", "#b4637a", "#56949f", "#ea9d34", "#286983", "#907aa9", "#d7827e", "#575279", "#9893a5",
            "#b4637a", "#56949f", "#ea9d34", "#286983", "#907aa9", "#d7827e", "#575279",
        ],
    }),
};

static AYU: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#0a0e14",
        fg: "#b3b1ad",
        cursor: "#e6b450",
        ansi: [
            "#0a0e14", "#ff3333", "#c2d94c", "#ff8f40", "#59c2ff", "#ffee99", "#95e6cb", "#b3b1ad", "#4d5566",
            "#ff3333", "#c2d94c", "#ff8f40", "#59c2ff", "#ffee99", "#95e6cb", "#b3b1ad",
        ],
    },
    light: Some(SchemeColors {
        bg: "#fafafa",
        fg: "#575f66",
        cursor: "#ff9940",
        ansi: [
            "#575f66", "#f51818", "#86b300", "#f2ae49", "#399ee6", "#a37acc", "#4cbf99", "#fafafa", "#8a9199",
            "#f51818", "#86b300", "#f2ae49", "#399ee6", "#a37acc", "#4cbf99", "#fafafa",
        ],
    }),
};

static KANAGAWA: Scheme = Scheme {
    dark: SchemeColors {
        bg: "#1f1f28",
        fg: "#dcd7ba",
        cursor: "#dcd7ba",
        ansi: [
            "#090618", "#c34043", "#76946a", "#c0a36e", "#7e9cd8", "#957fb8", "#6a9589", "#dcd7ba", "#727169",
            "#e82424", "#98bb6c", "#e6c384", "#7fb4ca", "#938aa9", "#7aa89f", "#c8c093",
        ],
    },
    light: None,
};
