//use std::fs::File;
//use std::io::{BufRead, BufReader};
#![allow(dead_code)]

mod constants;

use constants::*;

const PERMUTATION_LIMIT: usize = 1000;


fn main() {
    //let file = File::open("main.rs").unwrap();
    //let reader = BufReader::new(file);
    //for (index, line) in reader.lines().enumerate() {
    //    let line = line.unwrap();
    //    println!("{}  {}", index + 1, line);
    //}
}



//use std::mem::discriminant;

//impl std::fmt::Display for Chord {
//    fn write(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            Modifier::Shift => write!(f, "Shift"),
//            Modifier::Shift => write!(f, "Shift"),
//            Modifier::Shift => write!(f, "Shift"),
//        }
//    }
//}

//run: cargo test -- --nocapture
//#[test]
fn hello() {
    let file = r#"
#hello
|super {{, alt, ctrl, ctrl alt}} Return| {{$TERMINAL, alacritty, st, sakura}} -e tmux.sh open
|super shift {{, alt, ctrl, ctrl alt}} Return| {{$TERMINAL, alacritty, st, sakura}}
|super shift q|

# Main
|super space ; super w| $TERMINAL -e sh -c 'echo "nmcli"; echo "===="; sudo nmtui'; statusbar-startrefresh.sh
|super space ; super e| $TERMINAL -e emacs-sandbox.sh -P -O d "${EMACSINIT}"
|super space ; super a| $TERMINAL -e alsamixer; statusbar-startrefresh.sh
|super space ; super s| $TERMINAL -e syncthing -no-browser
|super space ; super z| $TERMINAL -e htop
|super space ; super {{m,n}}| $TERMINAL -e tmux.sh open '{{mw.sh,newsboat}}'


|super d| dmenu_run
|super h| dmenu
"#;
    println!("{}", Hotkey(vec![
        Chord { key: "a", modifiers: Mod::Shift | Mod::Ctrl },
        Chord { key: "b", modifiers: Mod::Shift.into() },
    ]));

    //parse(file);
    let size = file.split("\n|").count();
    let _output = Vec::<Shortcut>::with_capacity(size);
    //println!("{} {}", size, split(file.trim_start()).count());

    let trimmed_file = file.trim_start();
    // .unwrap_or('a') could be anything but whitespace
    let first_non_whitespace = trimmed_file.chars().next().unwrap_or('a');
    let first_non_whitespace = file.find(first_non_whitespace).unwrap_or(0);
    let start_row = file[0..first_non_whitespace].lines().count();
    //let final_row = split(file, 1).last().map(|(_, _, r)| r).unwrap_or(1);

    for (entry, _, _row) in split(trimmed_file, start_row) {
        //println!("{}: {:?}", r, entry);
        match entry.chars().next() {
            Some('|') => {
                //parse_entry(&entry['|'.len_utf8()..])
                //let after_start = &entry['|'.len_utf8()..];
                //if let Some(closing_pipe) = after_start.find('|') {
                //    let (hotkey, body) = after_start.split_at(closing_pipe);
                //    output.push(Shortcut {
                //        hotkey,
                //        action: vec![body],
                //    });
                //} else {
                //    panic!("Cannot find a closing pipe at {}", row);
                //}
            }
            Some('#') => {} // Skip comments
            Some(_) => panic!("Invalid"),
            None => {}
        }
        //println!("line: {:?}", entry);
    }
}

#[test]
fn parser() {
    //let line = r#"super {{space, x}} ; super {{w,y,z}} ; super {{a,b,c,d}}| $TERMINAL -e sh -c 'echo "nmcli"; echo "===="; sudo nmtui'; statusbar-startrefresh.sh"#;
    //println!("line: {:?}", line);
    //parse_entry(line);

    let line = r#"|super {{x, y}} ; super {{a,b}}|
        echo {{1,2,3,4}}
    "#;
    println!("{:#?}", parse_entry(line));
    //println!("{:?}", split_head_body_and_validate(&line['|'.len_utf8()..]));
}

fn parse_entry(entry: &str) -> Result<Vec<Shortcut>, ()> {
    //let head_end = after_first_pipe.find('|').unwrap_or(after_first_pipe.len());
    let after_first_pipe = if let Some('|') = entry.chars().next() {
        &entry['|'.len_utf8()..]
    } else {
        panic!("DEV: called 'parse_entry' on something not an shortcut entry")
    };
    let (head, body) = split_head_body_and_validate(after_first_pipe)?;
    println!("first {:?} {:?}", head, body);

    let field_count = SpanDelimiterSplit::new(head, 1, split_brackets).count();
    let permutation_field_totals = {
        let mut by_fields = Vec::with_capacity(field_count);
        by_fields.extend(SpanDelimiterSplit::new(head, 1, split_brackets)
            .map(|(_, delim, _)| delim.split(',').count()));
        by_fields
    };

    let permutation_count = permutation_field_totals.iter().product::<usize>();
    if permutation_count > PERMUTATION_LIMIT {
        panic!("Too many permutations");
    }

    let num_variants =
        enumerate_num_variants(permutation_field_totals, permutation_count);

    // Render "num_variants" into String's
    let string_variants =
        convert_to_string_variants(head, num_variants, permutation_count);

    let mut variants = Vec::with_capacity(permutation_count);
    for (i, head_variant) in string_variants.iter().enumerate() {
        let hotkey = parse_into_hotkey(head_variant).unwrap();
        // TODO: make this actually take hotkey
        variants.push(Shortcut {
            hotkey: hotkey.0[0].key.to_string(),
            action: select_body_variant(body, i, field_count),
        });
    }
    //Ok(())
    Ok(variants)
}


#[derive(Debug)]
enum HeadState {
    Head,
    Brackets,
}
#[derive(Debug)]
enum BodyState {
    Body,
    Brackets,
}

// This ensures all subsequent parsing does not need to return error
// Instead, subsequent `panic!(...)` indicate inconsitent grammar implementation
fn split_head_body_and_validate(after_first_pipe: &str) -> Result<(&str, &str), ()> {
    // Validate the head and calculate 'head_end'
    let mut head_end = 0;
    let mut chars = after_first_pipe.chars();
    let mut state = HeadState::Head;
    while let Some(ch) = chars.next() {
        match state {
            HeadState::Head => match ch {
                '|' => break, // thus 'head_end' is before closing pipe
                '{' => {
                    if let Some('{') = chars.next() {
                        head_end += '{'.len_utf8();
                        state = HeadState::Brackets;
                    } else {
                        panic!("Missing a second opening curly brace. Need '{{' to start an enumeration");
                    }
                }
                // TODO: check is valid key
                _ => {}
            }
            HeadState::Brackets => match ch {
                '\\' => {
                    panic!("You cannot escape characters with backslash '\\' in the hotkey definition portion");
                }
                '}' => {
                    if let Some('}') = chars.next() {
                        head_end += '}'.len_utf8(); // first '}' already added
                        state = HeadState::Head;
                    } else {
                        panic!("Missing a second closing curly brace. Need '}}' to close an enumeration");
                    }
                }
                _ => {}
            }
        }
        head_end += ch.len_utf8(); // Placed here so '|' breaks before adding
    }
    if head_end == after_first_pipe.len() {
        panic!("Shortcut hotkey definition still open, no closing '|' found");
    }
    let head_end = head_end; // No more changes to 'index'


    // Validate the body
    let mut state = BodyState::Body;
    while let Some(ch) = chars.next() {
        match state {
            BodyState::Body => match ch {
                '{' => {
                    if let Some('{') = chars.next() {
                        state = BodyState::Brackets;
                    }
                }
                _ => {}
            }
            BodyState::Brackets => match ch {
                '\\' => { chars.next(); }
                '}' => {
                    if let Some('}') = chars.next() {
                        state = BodyState::Body;
                    } else {
                        panic!("Missing a second closing curly brace. Need '}}' to close");
                    }
                }
                _ => {}
            }
        }
    }


    Ok((
        &after_first_pipe[0..head_end],
        &after_first_pipe[head_end+'|'.len_utf8()..],
    ))
}

use std::borrow::Cow;
fn select_body_variant(body: &str, variant_index: usize, field_count: usize) -> Vec<Cow<str>> {
    let mut body_variant = Vec::with_capacity(field_count + 1);
    let mut buffer = String::new();
    for (key, delim, _row) in SpanDelimiterSplit::new(body, 1, split_brackets) {
        body_variant.push(key.into());
        //println!("{:?} {:?}", key, delim);

        // Process the bracketed options ('delim')
        let delim = if delim.is_empty() {
            delim
        } else {
            &delim["{{".len()..]
        };

        // Basically a `delim.split(',')` but with escaping backslash
        // Additionally escaped newlines are ignored (similar to shellscript)
        // Push the delim when we get to the correct field
        let mut walker = delim.chars();
        let mut start = 0;
        let mut until = 0;
        let mut fields_visited = 0;
        while let Some(ch) = walker.next() {
            match ch {
                '\\' => {
                    buffer.push_str(&delim[start..until]);
                    let escaped = walker.next().unwrap();
                    if escaped != '\n' {
                        buffer.push(escaped);
                    }
                    until += '\\'.len_utf8() + escaped.len_utf8();
                    start = until;
                }
                ',' | '}' => {
                    if variant_index == fields_visited {
                        buffer.push_str(&delim[start..until]);
                        body_variant.push(buffer.split_off(0).into());
                        break;
                    }
                    debug_assert_eq!(','.len_utf8(), '}'.len_utf8());
                    start = until + ','.len_utf8();
                    until = start;
                    fields_visited += 1;
                    buffer.clear();
                }
                c => until += c.len_utf8(),
            }
        }
    }
    body_variant
}

fn parse_into_hotkey(head: &str) -> Result<Hotkey, ()> {
    let mut split = SpanDelimiterSplit::new(head, 1, |substr| {
        if let Some(start) = substr.find(&SEPARATOR[..]) {
            let mut after_delimit = substr[start..].chars();
            while let Some(ch) = after_delimit.next()  {
                if !SEPARATOR.iter().any(|c| *c == ch) {
                    break
                }
            }
            // SAFETY: `after_delimit.next()` called so can always `- 1`
            // At max len: `after_delimit.as_str().len() + 1 == start`
            start..substr.len() - after_delimit.as_str().len() - 1
        } else {
            substr.len()..substr.len()
        }
    }).peekable();

    let mut modifiers = 0;
    let mut key: Option<&str> = None;
    // Hotkeys likely 1-4 in chords in length, 'with_capacity()' unneeded
    let mut chords = Vec::new();
    while let Some((field, delim, _row)) = split.next() {
        match field  {
            "shift" => modifiers |= Mod::Shift as Modifiers,
            "super" => modifiers |= Mod::Super as Modifiers,
            "ctrl" =>  modifiers |= Mod::Ctrl as Modifiers,
            "alt" =>   modifiers |= Mod::Alt as Modifiers,
            _ if key.is_some() => panic!("Key already defined"),
            _ if KEYCODES.contains(&field) => key = Some(field),
            _ => panic!("Invalid key {}", field),
        }

        if delim.contains(';') || split.peek().is_none() {
            if let Some(code) = key {
                chords.push(Chord { key: code, modifiers })
            } else {
                panic!("No key set")
            }
            modifiers = 0;
            key = None;
        }
    }
    //chords.iter().for_each(|s| println!("Shortcut {}", s));
    Ok(Hotkey(chords))
}


fn enumerate_num_variants(field_totals: Vec<usize>, permutation_count: usize) -> Vec<Vec<usize>> {
    // Build up enumerations for picking variants in 'head'
    // TODO: Maybe a way to enumerate with modulus?
    let field_count = field_totals.len();
    let mut num_variants = Vec::with_capacity(permutation_count);
    let mut variant = Vec::with_capacity(field_count);
    variant.resize(field_count, 0);
    let last = field_count - 1;
    // Basically doing a carry-add with bases of 'permutations[..]'
    for _ in 0..permutation_count - 1 {
        num_variants.push(variant.clone());

        variant[last] += 1;
        // Calculate the carry
        // Order is enumerate the last first
        for i in (0..field_count-1).rev() { // will not overflow so can skip 0
            if variant[i+1] >= field_totals[i+1] {
                variant[i+1] = 0;
                variant[i] += 1;
            }
        }
    }
    num_variants.push(variant);
    //num_variants.iter().for_each(|v| println!("{:?}", v));
    num_variants
}
fn convert_to_string_variants(
    head: &str,
    num_variants: Vec<Vec<usize>>,
    permutation_count: usize,
) -> Vec<String> {
    let mut rendered_variants = Vec::with_capacity(permutation_count);
    for permutation in num_variants {
        let mut variant = String::with_capacity(head.len());
        let split = SpanDelimiterSplit::new(head, 1, split_brackets);
        for (i, (unbracketed, brackets, _r)) in split.enumerate() {
            let inside = "{{".len()..brackets.len() - "}}".len();
            variant.push_str(unbracketed);
            let choice = brackets[inside].split(',').nth(permutation[i]);
            variant.push_str(choice.unwrap());
        }
        rendered_variants.push(variant);

    }
    //rendered_variants.iter().for_each(|v| println!("{:?}", v));
    rendered_variants
}


use std::ops::Range;


// Split with delimiter of '{{..}}'
// Backslash escaping is allowed within the delimiter
fn split_brackets(substr: &str) -> Range<usize> {
    let len = substr.len();
    let (start, mut close) = if let Some(i) = substr.find("{{") {
        (i, i + "{{".len())
    } else {
        (len, len)
    };
    //if start > substr.find("}}").unwrap_or(len) {
    //    panic!("DEV: Validation did not catch '}}' found without an opening '{{'");
    //}

    let mut chars = substr[close..].chars();
    while let Some(ch) = chars.next() {
        close += ch.len_utf8();
        match ch {
            '\\' => {
                close += chars.next().map(|c| c.len_utf8()).unwrap_or(0);
            }
            '}' => {
                if let Some(c) = chars.next() {
                    close += c.len_utf8();
                    if c == '}' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    start..close
}


fn split(source: &str, start_row: usize) -> SpanDelimiterSplit {
    SpanDelimiterSplit {
        buffer: source,
        row: start_row,
        delimit_by: |substr| {
            let len = substr.len();
            let l = '\n'.len_utf8();
            let hotkey = substr.find("\n|").map(|i| i + l).unwrap_or(len);
            let comment = substr.find("\n#").map(|i| i + l).unwrap_or(len);
            if hotkey < comment {
                hotkey..hotkey
            } else {
                comment..comment
            }
        }
    }
}

struct SpanDelimiterSplit<'a> {
    buffer: &'a str,
    row: usize,
    delimit_by: fn(&str) -> Range<usize>,
}
impl<'a> SpanDelimiterSplit<'a> {
    fn new(s: &'a str, start_row: usize, f: fn (&str) -> Range<usize>) -> Self {
        Self {
            buffer: s,
            row: start_row,
            delimit_by: f,
        }
    }
}

impl<'a> Iterator for SpanDelimiterSplit<'a> {
    type Item = (&'a str, &'a str, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let rel_delim = (self.delimit_by)(self.buffer);
        let buffer_len = self.buffer.len();
        if buffer_len > 0 {
            let field = &self.buffer[0..rel_delim.start];
            let delimiter = &self.buffer[rel_delim.start..rel_delim.end];
            let row = self.row;
            self.row += field.lines().count();
            self.buffer = &self.buffer[rel_delim.end..];
            Some((field, delimiter, row))
        } else {
            None

        }
    }
}

