pub type Keymap<'a> = (&'a str, &'a str);

pub const PRIMARY: [Keymap; 10] = [
    ("j/k", "Navigate"),
    ("n/p", "Next/Prev"),
    ("Enter", "Play"),
    ("Space", "Toggle"),
    ("+/-", "Volume"),
    ("s", "Shuffle"),
    ("r", "Repeat"),
    ("←/→", "Seek"),
    ("/", "Search"),
    ("q", "Quit"),
];

pub const SEARCH: [Keymap; 3] = [
    ("Esc", "Exit"),
    ("Enter", "Confirm"),
    ("←/→", "Move cursor"),
];
