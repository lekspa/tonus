pub type Keymap<'a> = (&'a str, &'a str);

pub const GENERAL: [Keymap; 5] = [
    ("j, Down     ", "Move down"),
    ("k, Up       ", "Move up"),
    ("/           ", "Show search"),
    ("?           ", "Toggle keymaps"),
    ("q           ", "Quit"),
];

pub const PLAYBACK: [Keymap; 10] = [
    ("n           ", "Play next track"),
    ("p           ", "Play previous track"),
    ("Enter       ", "Play selected track"),
    ("Space       ", "Toggle pause play"),
    ("+, =        ", "Increase volume"),
    ("-           ", "Decrease volume"),
    ("s           ", "Toggle shuffle"),
    ("r           ", "Toggle repeat"),
    ("Ctrl-Right  ", "Seek forward"),
    ("Ctrl-Left   ", "Seek backwards"),
];

pub const SEARCH: [Keymap; 4] = [
    ("Esc         ", "Reset and close"),
    ("Enter       ", "Confirm query"),
    ("Left        ", "Move cursor left"),
    ("Right       ", "Move cursor right"),
];
