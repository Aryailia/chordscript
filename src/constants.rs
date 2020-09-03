//run: time cargo test -- --nocapture

use std::borrow::Cow;

#[derive(Debug)]
pub struct Shortcut<'a> {
    pub hotkey: String, // TODO: change to Hotkey
    pub action: Vec<Cow<'a, str>>,
}

#[derive(Debug)]
pub struct Hotkey(pub Vec<Chord>);

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let chords = &self.0;
        if !chords.is_empty() {
            write!(f, "{}", chords[0])?;
            for i in 1..chords.len() {
                write!(f, " ; {}", chords[i])?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Chord {
    pub key: Key,

    // TODO: Make this into a bit field?
    pub modifiers: u16,
}
impl std::fmt::Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let modifier_bitfield = self.modifiers;
        for i in 0..MOD_SIZE {
            if modifier_bitfield & (1 << i) > 0 {
                write!(f, "{:?} + ", NUM_TO_MOD[i as usize].clone())?;
            }
        }
        write!(f, "{:?}", self.key)
    }
}

pub type Modifiers = u16;

macro_rules! enum_mod {
    ($type:ty { $($variant:ident,)* }) => {
        // This increments from 0 by 1 automatically (defined in rust reference)
        enum _ModCounter {
            $($variant,)*
            _Size,
        }

        #[derive(Debug, Clone)]
        #[allow(dead_code)]
        //#[repr($type)]
        pub enum Mod {
            $($variant = 1 << _ModCounter::$variant as $type,)*
        }

        impl From<Mod> for $type {
            fn from(me: Mod) -> $type {
                me as $type
            }
        }
        impl std::ops::BitOr for Mod {
            type Output = $type;
            fn bitor(self, rhs: Self) -> $type {
                self as $type | rhs as $type
            }
        }
        pub const MOD_SIZE: $type = _ModCounter::_Size as $type;
        pub const NUM_TO_MOD: [Mod; MOD_SIZE as usize] = [$(Mod::$variant,)*];
    };
}

enum_mod! {
    Modifiers {
        Shift,
        Super,
        Ctrl,
        Alt,
    }
}

// https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt
// These contain no semantic meaning in head
pub const SEPARATOR: [char; 26] = [
    '+', // rest is whitespace as defined by unicode
    '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{0085}', '\u{00a0}',
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}',
    '\u{3000}',
];

macro_rules! declare_keycodes {
    ($($keystr:literal = $variant:ident,)*) => {
        enum _KeySize {
            $($variant,)*
            Size,
        }

        #[derive(Clone, Debug)]
        pub enum Key {
            $($variant,)*
        }

        // TODO: replace this with phf::Map?
        // Might be too bloated for minor speedup
        pub const KEYSTRS: [&'static str; _KeySize::Size as usize] = [
            $($keystr,)*
        ];

        pub const KEYCODES: [Key; _KeySize::Size as usize] = [
            $(Key::$variant,)*
        ];

        pub const KEYSTR_MAX_LEN: usize = {
            let mut max = 0;
            $(if $keystr.len() > max {
                max = $keystr.len();
            })*
            max
        };

        pub const KEYSTR_LEN_TO_CHECK: [bool; KEYSTR_MAX_LEN] = {
            let mut temp = [false; KEYSTR_MAX_LEN];
            $(temp[$keystr.len() - 1] = true;)*
            temp
        };
    };
}
declare_keycodes! {
    "space"   = Space,
    "a"       = A,
    "b"       = B,
    "c"       = C,
    "d"       = D,
    "e"       = E,
    "f"       = F,
    "g"       = G,
    "h"       = H,
    "i"       = I,
    "j"       = J,
    "k"       = K,
    "l"       = L,
    "m"       = M,
    "n"       = N,
    "o"       = O,
    "p"       = P,
    "q"       = Q,
    "r"       = R,
    "s"       = S,
    "t"       = T,
    "u"       = U,
    "v"       = V,
    "w"       = W,
    "x"       = X,
    "y"       = Y,
    "z"       = Z,
    "0"       = Zero,
    "1"       = One,
    "2"       = Two,
    "3"       = Three,
    "4"       = Four,
    "5"       = Five,
    "6"       = Six,
    "7"       = Seven,
    "8"       = Eight,
    "9"       = Nine,
    "Return"  = Return,
    "Comma"   = Comma,
}

#[test]
fn unique_keys() {
    use std::mem::discriminant;
    for (i, k1) in KEYSTRS.iter().enumerate() {
        for k2 in KEYSTRS[i + 1..].iter() {
            assert!(k1 != k2, "{:?} is duplicated", k1);
        }
    }
    for (i, k1) in KEYCODES.iter().enumerate() {
        for k2 in KEYCODES[i + 1..].iter() {
            assert!(discriminant(k1) != discriminant(k2), "\"{:?}\" is duplicated", k1);
        }
    }
}


