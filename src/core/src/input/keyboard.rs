use std::cell::RefCell;

// Copy from SDL
#[derive(Copy, Clone, Debug)]
pub enum Key {
    Unknown = 0,

    /**
     *  \name Usage page 0x07
     *
     *  These values are from usage page 0x07 (USB keyboard page).
     */
    /* @{ */

    A = 4,
    B = 5,
    C = 6,
    D = 7,
    E = 8,
    F = 9,
    G = 10,
    H = 11,
    I = 12,
    J = 13,
    K = 14,
    L = 15,
    M = 16,
    N = 17,
    O = 18,
    P = 19,
    Q = 20,
    R = 21,
    S = 22,
    T = 23,
    U = 24,
    V = 25,
    W = 26,
    X = 27,
    Y = 28,
    Z = 29,

    Num1 = 30,
    Num2 = 31,
    Num3 = 32,
    Num4 = 33,
    Num5 = 34,
    Num6 = 35,
    Num7 = 36,
    Num8 = 37,
    Num9 = 38,
    Num0 = 39,

    Return = 40,
    Escape = 41,
    Backspace = 42,
    Tab = 43,
    Space = 44,

    Minus = 45,
    Equals = 46,
    LeftBracket = 47,
    RightBracket = 48,

    /// Located at the lower left of the return
    /// key on ISO keyboards and at the right end
    /// of the QWERTY row on ANSI keyboards.
    /// Produces REVERSE SOLIDUS (backslash) and
    /// VERTICAL LINE in a US layout, REVERSE
    /// SOLIDUS and VERTICAL LINE in a UK Mac
    /// layout, NUMBER SIGN and TILDE in a UK
    /// Windows layout, DOLLAR SIGN and POUND SIGN
    /// in a Swiss German layout, NUMBER SIGN and
    /// APOSTROPHE in a German layout, GRAVE
    /// ACCENT and POUND SIGN in a French Mac
    /// layout, and ASTERISK and MICRO SIGN in a
    /// French Windows layout.
    BackSlash = 49,

    /// ISO USB keyboards actually use this code
    /// instead of 49 for the same key, but all
    /// OSes I've seen treat the two codes
    /// identically. So, as an implementor, unless
    /// your keyboard generates both of those
    /// codes and your OS treats them differently,
    /// you should generate Key::BackSlash
    /// instead of this code. As a user, you
    /// should not rely on this code because we
    /// will never generate it with most (all?)
    /// keyboards.
    NonUsHash = 50,

    Semicolon = 51,
    Apostrophe = 52,

    /// Located in the top left corner (on both ANSI
    /// and ISO keyboards). Produces GRAVE ACCENT and
    /// TILDE in a US Windows layout and in US and UK
    /// Mac layouts on ANSI keyboards, GRAVE ACCENT
    /// and NOT SIGN in a UK Windows layout, SECTION
    /// SIGN and PLUS-MINUS SIGN in US and UK Mac
    /// layouts on ISO keyboards, SECTION SIGN and
    /// DEGREE SIGN in a Swiss German layout (Mac:
    /// only on ISO keyboards), CIRCUMFLEX ACCENT and
    /// DEGREE SIGN in a German layout (Mac: only on
    /// ISO keyboards), SUPERSCRIPT TWO and TILDE in a
    /// French Windows layout, COMMERCIAL AT and
    /// NUMBER SIGN in a French Mac layout on ISO
    /// keyboards, and LESS-THAN SIGN and GREATER-THAN
    /// SIGN in a Swiss German, German, or French Mac
    /// layout on ANSI keyboards.
    Grave = 53,

    Comma = 54,
    Period = 55,
    Slash = 56,

    CapsLock = 57,

    F1 = 58,
    F2 = 59,
    F3 = 60,
    F4 = 61,
    F5 = 62,
    F6 = 63,
    F7 = 64,
    F8 = 65,
    F9 = 66,
    F10 = 67,
    F11 = 68,
    F12 = 69,

    PrintScreen = 70,
    ScrollLock = 71,
    Pause = 72,
     /// insert on PC, help on some Mac keyboards (but
     /// does send code 73, not 117)
    Insert = 73,

    Home = 74,
    PageUp = 75,
    Delete = 76,
    End = 77,
    PageDown = 78,
    Right = 79,
    Left = 80,
    Down = 81,
    Up = 82,

    /// num lock on PC, clear on Mac keyboards
    NumLockClear = 83,

    KpDivide = 84,
    KpMultiply = 85,
    KpMinus = 86,
    KpPlus = 87,
    KpEnter = 88,
    Kp1 = 89,
    Kp2 = 90,
    Kp3 = 91,
    Kp4 = 92,
    Kp5 = 93,
    Kp6 = 94,
    Kp7 = 95,
    Kp8 = 96,
    Kp9 = 97,
    Kp0 = 98,
    KpPeriod = 99,

    /// This is the additional key that ISO
    /// keyboards have over ANSI ones,
    /// located between left shift and Y.
    /// Produces GRAVE ACCENT and TILDE in a
    /// US or UK Mac layout, REVERSE SOLIDUS
    /// (backslash) and VERTICAL LINE in a
    /// US or UK Windows layout, and
    /// LESS-THAN SIGN and GREATER-THAN SIGN
    /// in a Swiss German, German, or French
    /// layout.
    NonUsBackSlash = 100,
    /// windows contextual menu, compose
    Application = 101,

    /// The USB document says this is a status flag,
    /// not a physical key - but some Mac keyboards
    /// do have a power key.
    Power = 102,

    KpEquals = 103,
    F13 = 104,
    F14 = 105,
    F15 = 106,
    F16 = 107,
    F17 = 108,
    F18 = 109,
    F19 = 110,
    F20 = 111,
    F21 = 112,
    F22 = 113,
    F23 = 114,
    F24 = 115,
    Execute = 116,
    Help = 117,
    Menu = 118,
    Select = 119,
    Stop = 120,
    Again = 121,   /**< redo */
    Undo = 122,
    Cut = 123,
    Copy = 124,
    Paste = 125,
    Find = 126,
    Mute = 127,
    VolumeUp = 128,
    VolumeDown = 129,
/* not sure whether there's a reason to enable these */
/*     LockIngCapsLock = 130,  */
/*     LockIngNumLock = 131, */
/*     LockIngScrollLock = 132, */
    KpComma = 133,
    KpEqualSas400 = 134,

    /// used on Asian keyboards, see footnotes in USB doc
    International1 = 135,
    International2 = 136,
    International3 = 137,
    International4 = 138,
    International5 = 139,
    International6 = 140,
    International7 = 141,
    International8 = 142,
    International9 = 143,
    /// Hangul/English toggle
    Lang1 = 144,
    /// Hanja conversion
    Lang2 = 145,
    /// Katakana
    Lang3 = 146,
    /// Hiragana
    Lang4 = 147,
    /// Zenkaku/Hankaku
    Lang5 = 148,
    /// reserved
    Lang6 = 149,
    /// reserved
    Lang7 = 150,
    /// reserved
    Lang8 = 151,
    /// reserved
    Lang9 = 152,

    /// Erase-Eaze
    AltErase = 153,
    SysReq = 154,
    Cancel = 155,
    Clear = 156,
    Prior = 157,
    Return2 = 158,
    Separator = 159,
    Out = 160,
    Oper = 161,
    ClearAgain = 162,
    Crsel = 163,
    Exsel = 164,

    Kp00 = 176,
    Kp000 = 177,
    ThousandsSeparator = 178,
    DecimalSeparator = 179,
    CurrencyUnit = 180,
    CurrencySubunit = 181,
    KpLeftParen = 182,
    KpRightParen = 183,
    KpLeftBrace = 184,
    KpRightBrace = 185,
    KpTab = 186,
    KpBackspace = 187,
    KpA = 188,
    KpB = 189,
    KpC = 190,
    KpD = 191,
    KpE = 192,
    KpF = 193,
    KpXor = 194,
    KpPower = 195,
    KpPercent = 196,
    KpLess = 197,
    KpGreater = 198,
    KpAmpersand = 199,
    KpDblAmpersand = 200,
    KpVerticalBar = 201,
    KpDblVerticalBar = 202,
    KpColon = 203,
    KpHash = 204,
    KpSpace = 205,
    KpAt = 206,
    KpExclam = 207,
    KpMemStore = 208,
    KpMemRecall = 209,
    KpMemClear = 210,
    KpMemAdd = 211,
    KpMemSubtract = 212,
    KpMemMultiply = 213,
    KpMemDivide = 214,
    KpPlusMinus = 215,
    KpClear = 216,
    KpClearEntry = 217,
    KpBinary = 218,
    KpOctal = 219,
    KpDecimal = 220,
    KpHexadecimal = 221,

    LCtrl = 224,
    LShift = 225,
    /// alt, option
    LAlt = 226,
    /// windows, command (apple), meta
    LGui = 227,
    RCtrl = 228,
    RShift = 229,
    /// alt gr, option
    RAlt = 230,
    /// windows, command (apple), meta
    RGui = 231,

    /// I'm not sure if this is really not covered
    /// by any of the above, but since there's a
    /// special KMOD_MODE for it I'm adding it here
    Mode = 257,

    /* @} *//* Usage page 0x07 */

    /**
     *  \name Usage page 0x0C
     *
     *  These values are mapped from usage page 0x0C (USB consumer page).
     */
    /* @{ */

    AudioNext = 258,
    AudioPrev = 259,
    AudioStop = 260,
    AudioPlay = 261,
    AudioMute = 262,
    MediaSelect = 263,
    Www = 264,
    Mail = 265,
    Calculator = 266,
    Computer = 267,
    AcSearch = 268,
    AcHome = 269,
    AcBack = 270,
    AcForward = 271,
    AcStop = 272,
    AcRefresh = 273,
    AcBookmarks = 274,

    /* @} *//* Usage page 0x0C */

    /**
     *  \name Walther keys
     *
     *  These are values that Christian Walther added (for mac keyboard?).
     */
    /* @{ */

    BrightnessDown = 275,
    BrightnessUp = 276,
    /// display mirroring/dual display switch, video mode switch
    DisplaySwitch = 277,

    KbdillumToggle = 278,
    KbdillumDown = 279,
    KbdillumUu = 280,
    Eject = 281,
    Sleep = 282,

    App1 = 283,
    App2 = 284,

    /* @} *//* Walther keys */

    /* Add any other keys here. */


    /// not a key, just marks the number of scancodes
    /// for array bounds
    Count = 512
}

pub struct Keyboard {
    keys: RefCell<[KeyState; Key::Count as usize]>,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            keys: RefCell::new([KeyState::new(); Key::Count as usize]),
        }
    }

    pub fn down(&self, key: Key) -> bool {
        self.keys.borrow()[key as usize].is_down
    }

    pub fn up(&self, key: Key) -> bool {
        !self.keys.borrow()[key as usize].is_down
    }

    pub fn press(&self, key: Key) -> bool {
        !self.keys.borrow()[key as usize].was_down && self.keys.borrow()[key as usize].is_down
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.keys.borrow()[key as usize].was_down && !self.keys.borrow()[key as usize].is_down
    }

    pub fn set_down(&self, key: Key) {
        self.keys.borrow_mut()[key as usize].is_down = true;
    }

    pub fn set_up(&self, key: Key) {
        self.keys.borrow_mut()[key as usize].is_down = false;
    }

    pub fn update(&self) {
        for key in self.keys.borrow_mut().iter_mut() {
            key.was_down = key.is_down;
        }
    }
}

#[derive(Copy, Clone)]
struct KeyState {
    was_down: bool,
    is_down: bool,
}

impl KeyState {
    pub fn new() -> KeyState {
        KeyState { was_down: false, is_down: false }
    }
}

thread_local!(static KEYBOARD: Keyboard = Keyboard::new());

pub fn down(key: Key) -> bool {
    KEYBOARD.with(|keyboard| keyboard.down(key))
}

pub fn up(key: Key) -> bool {
    KEYBOARD.with(|keyboard| keyboard.up(key))
}

pub fn press(key: Key) -> bool {
    KEYBOARD.with(|keyboard| keyboard.press(key))
}

pub fn pressed(key: Key) -> bool {
    KEYBOARD.with(|keyboard| keyboard.pressed(key))
}

pub fn set_down(key: Key) {
    KEYBOARD.with(|keyboard| keyboard.set_down(key))
}

pub fn set_up(key: Key) {
    KEYBOARD.with(|keyboard| keyboard.set_up(key))
}

pub fn update() {
    KEYBOARD.with(|keyboard| keyboard.update())
}
