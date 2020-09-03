//use std::fs::File;
//use std::io::{BufRead, BufReader};
#![allow(dead_code)]

mod constants;

use constants::*;

const PERMUTATION_LIMIT: usize = 1000;


// Terminology
// 
// A set I refering to the '{{}}' construct. These represent enumerations
// Both the head and the body can have multiple sets
//
// Example ('+' and ' ' are both optional in the head)
//   Entry:  |super {{a,b}}; ctrl + {{1,2}} ; ctrl shift b| echo
//   Head:   |super {{a,b}}; ctrl + {{1,2}} ; ctrl shift b|
//   Body:   echo
//   Hotkey: super + a ; ctrl + 1 ; ctrl + shift + b
//           super + a ; ctrl + 2 ; ctrl + shift + b
//           super + b ; ctrl + 1 ; ctrl + shift + b
//           super + b ; ctrl + 2 ; ctrl + shift + b
//   Set:    {{a, b}}
//           {{1, 2}}
//   Chord:  super + a
//           ctrl + 1
//           ctrl + shift + b
//           ...
// A 'shortcut' is then the hotkey variant + body variant together ready
// to be rendered in the target format


fn main() {
    //let file = File::open("main.rs").unwrap();
    //let reader = BufReader::new(file);
    //for (index, line) in reader.lines().enumerate() {
    //    let line = line.unwrap();
    //    println!("{}  {}", index + 1, line);
    //}
}

//run: cargo test -- --nocapture
//    let file = r#"
//#hello
//|super {{, alt, ctrl, ctrl alt}} Return| {{$TERMINAL, alacritty, st, sakura}} -e tmux.sh open
//|super shift {{, alt, ctrl, ctrl alt}} Return| {{$TERMINAL, alacritty, st, sakura}}
//|super shift q|
//
//# Main
//|super space ; super w| $TERMINAL -e sh -c 'echo "nmcli"; echo "===="; sudo nmtui'; statusbar-startrefresh.sh
//|super space ; super e| $TERMINAL -e emacs-sandbox.sh -P -O d "${EMACSINIT}"
//|super space ; super a| $TERMINAL -e alsamixer; statusbar-startrefresh.sh
//|super space ; super s| $TERMINAL -e syncthing -no-browser
//|super space ; super z| $TERMINAL -e htop
//|super space ; super {{m,n}}| $TERMINAL -e tmux.sh open '{{mw.sh,newsboat}}'
//
//
//|super d| dmenu_run
//|super h| dmenu
//"#;

#[derive(Debug)]
enum State {
    Head,
    HeadBrackets,
    Body,
    BodyBrackets,
}

#[derive(Debug)]
struct UnparsedEntry<'a> {
    head: &'a str,
    body: &'a str,
    head_set_count: usize,
    body_set_count: usize,
    permutation_count: usize,
    row: usize,
}

#[derive(Debug)]
struct EntryBlobMetadata<'a> {
    entries: Vec<UnparsedEntry<'a>>,
    max_head_set_count: usize,
    max_body_set_count: usize,
    max_permutation_count: usize,
}

// TODO: test when body has more sets than head
// e.g. |{{a,b,c}};{{1,2,3,4}}| {{a,b}} {{e, f}} {{g,h}}
// This has 12 vs 8 permutations, the last 4 permutations will all have the
// same body variant but
// The reverse case (more body variants) than 

use std::cmp::max;

impl<'a> EntryBlobMetadata<'a> {
    fn push_entry(&mut self, entry: UnparsedEntry<'a>) {
        self.max_head_set_count = max(self.max_head_set_count, entry.head_set_count);
        self.max_body_set_count = max(self.max_body_set_count, entry.body_set_count);
        self.max_permutation_count = max(self.max_permutation_count, entry.permutation_count);
        self.entries.push(entry);
    }
}

impl<'a> FirstPass<'a> {
    fn step_init_until_first(source: &str) -> Result<(&str, usize), StepError> {
        let mut row = 0;
        let mut start = source.len();
        for line in source.lines() {
            row += 1;
            if let Some('|') = line.chars().next() {
                let one = 'l'.len_utf8();
                start = line.as_ptr() as usize - source.as_ptr() as usize + one;
                break;
            }
            match line.trim_start().chars().next() {
                Some('#') => {}
                Some(_) => return Err("Lines can only be a comment (first non-whitespace character is '#') or whitespace before the first entry (first character in line is '|')".into()),
                None => {}
            }
        }

        Ok((&source[start..], row))
    }

    #[inline]
    fn step_head(&mut self, ch: char) -> PassOutput {
        match ch {
            '|' => {
                let base = self.original.as_ptr() as usize;
                let offset = self.entry.head.as_ptr() as usize - base;
                self.entry.head = &self.original[offset..self.walker.prev];
                self.entry.body = &self.original[self.walker.post..];
                self.change_state(State::Body)?; // Call last
                                                 //println!("==={:?}===\n{:?}", self.entry.head, self.entry.body);
            }
            '{' => {
                if let Some('{') = self.walker.next() {
                    // Want these three things on
                    self.change_state(State::HeadBrackets)?; // Call last
                } else {
                    return Err(
                        "Missing a second opening curly brace. Need '{{' to start an enumeration"
                            .into(),
                    );
                }
            }
            ',' => return Err("Unexpected comma ','. Type 'comma' for the key, ';' for a chord separator. ',' only has meaning inside an enumeration group '{{..}}'".into()),
            ';' => {
                self.walker.eat_separator();
                self.key_start_index = self.walker.post;
            }
            _ if SEPARATOR.contains(&ch) => {
                self.walker.eat_separator();
                self.key_start_index = self.walker.post;
            }
            _ => {
                let start = self.key_start_index;
                let key = &self.original[start..self.walker.post];
                if key.len() > KEYSTR_MAX_LEN {
                    panic!("Invalid keycode {:?}", key);
                }
                // Key validation check will happen when we parse the key
                // so we do since we allocate at that time
            }
        }
        Ok(())
    }

    #[inline]
    fn step_head_brackets(&mut self, ch: char) -> PassOutput {
        match ch {
            '|' => return Err("Unexpected bar '|'. Close the enumeration first with '}}'".into()),
            '\\' => {
                return Err("You cannot escape characters with backslash '\\' in the hotkey definition portion".into());
            }
            ',' => self.head_set_member_end(),
            '}' => {
                if let Some('}') = self.walker.next() {
                    self.change_state(State::Head)?; // Call last
                } else {
                    return Err(
                        "Missing a second closing curly brace. Need '}}' to close an enumeration"
                            .into(),
                    );
                }
            }
            _ if SEPARATOR.contains(&ch) => {
                self.walker.eat_whitespace();
                self.key_start_index = self.walker.post;
            }
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn step_body(&mut self, ch: char) -> PassOutput {
        match (ch, self.walker.peek()) {
            ('\n', Some('|')) => {
                self.walker.next();
                let base = self.original.as_ptr() as usize;
                let offset = self.entry.body.as_ptr() as usize - base;
                self.entry.body = &self.original[offset..self.walker.prev];
                //println!("==={}===\n{:?}", self.entry.head, self.entry.body);

                let new_entry = UnparsedEntry {
                    head: &self.original[self.walker.post..],
                    body: &self.original[self.walker.post..],
                    head_set_count: 0,
                    body_set_count: 0,
                    permutation_count: 1,
                    row: self.walker.row,
                };
                self.metadata
                    .push_entry(replace(&mut self.entry, new_entry));

                self.change_state(State::Head)?; // Call last
            }
            ('{', Some('{')) => self.change_state(State::BodyBrackets)?, // Call last
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn step_body_brackets(&mut self, ch: char) -> PassOutput {
        match ch {
            '\\' => {
                self.walker.next();
            }
            ',' => self.body_set_member_end()?,
            '}' => {
                if let Some('}') = self.walker.next() {
                    self.change_state(State::Body)?; // Call last
                } else {
                    return Err("Missing a second closing curly brace. Need '}}' to close. If you want a '}' as output, escape it with backslash like '\\}'".into());
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn head_set_start(&mut self) {
        self.walker.eat_separator();
        self.key_start_index = self.walker.post;
        self.head_set_size = 0;
    }

    #[inline]
    fn head_set_member_end(&mut self) {
        self.walker.eat_separator();
        self.key_start_index = self.walker.post;
        self.head_set_size += 1;
    }

    #[inline]
    fn head_set_close(&mut self) -> Result<(), StepError> {
        self.head_set_size += 1;
        self.entry.permutation_count *= self.head_set_size;
        self.entry.head_set_count += 1;
        //println!("group_end {:?}", self.entry.permutation_count, )
        if self.entry.permutation_count > PERMUTATION_LIMIT {
            return Err("Too many permutations for <line>".into());
        } else {
            Ok(())
        }
    }

    #[inline]
    fn body_set_start(&mut self) {
        self.body_set_size = 0;
    }

    #[inline]
    fn body_set_member_end(&mut self) -> Result<(), StepError> {
        self.body_set_size += 1;
        let permutation_test= self.body_set_size * self.body_permutation_count;
        if permutation_test > self.entry.permutation_count {
            Err(
                "This body for (TODO) needs more options than there are hotkey permutations for"
                    .into(),
            )
        } else {
            Ok(())
        }
    }
    #[inline]
    fn body_set_close(&mut self) -> Result<(), StepError> {
        self.body_set_member_end()?; // adds to 'self.body_set_size'
        self.entry.body_set_count += 1;
        self.body_permutation_count *= self.body_set_size;
        Ok(())
    }

    fn change_state(&mut self, target: State) -> Result<(), StepError> {
        // From 'self.state' to 'target'
        match (&self.state, &target) {
            (_, State::HeadBrackets) => self.head_set_start(),
            (State::HeadBrackets, _) => self.head_set_close()?,


            (_, State::BodyBrackets) => self.body_set_start(),
            (State::BodyBrackets, _) => self.body_set_close()?,

            (_, State::Head) => {
                self.walker.eat_separator();
                self.key_start_index = self.walker.post;
            }

            // TODO: Maybe change to compile-time state transition validation
            // See 'pretty state machines' blog post
            _ => {} // Maybe panic on invalid transitions? Kind of unnecessary
        }
        self.state = target;
        Ok(())
    }
}

use std::mem::replace;

type StepError = String;
type PassOutput<'a> = Result<(), StepError>;
struct FirstPass<'a> {
    original: &'a str,
    walker: CharsWithIndex<'a>,
    state: State,

    key_start_index: usize,
    head_set_size: usize,
    body_set_size: usize,
    body_permutation_count: usize,

    entry: UnparsedEntry<'a>,
    metadata: EntryBlobMetadata<'a>,
}
fn first_pass(source: &str) -> Result<EntryBlobMetadata, String> {
    let (text, start_row) = FirstPass::step_init_until_first(source)?;
    let mut fsm = FirstPass {
        original: text,
        walker: CharsWithIndex::new(text, start_row),
        state: State::Head,

        key_start_index: 0,
        head_set_size: 0,
        body_set_size: 0,
        body_permutation_count: 0,

        entry: UnparsedEntry {
            head: text,
            body: text,
            head_set_count: 0,
            body_set_count: 0,
            permutation_count: 1,
            row: start_row,
        },
        metadata: EntryBlobMetadata {
            entries: Vec::with_capacity(text.split("\n|").count()),
            max_head_set_count: 0,
            max_body_set_count: 0,
            max_permutation_count: 1,
        },
    };

    while let Some(ch) = fsm.walker.next() {
        match fsm.state {
            State::Head => fsm.step_head(ch)?,
            State::HeadBrackets => fsm.step_head_brackets(ch)?,
            State::Body => fsm.step_body(ch)?, // This may push
            State::BodyBrackets => fsm.step_body_brackets(ch)?,
        };
    }
    if let State::HeadBrackets | State::BodyBrackets = fsm.state {
        return Err("Brackets not closed. Expected a '}}'".into());
    }
    let last = fsm.entry;
    if !last.head.is_empty() {
        fsm.metadata.push_entry(last);
    }
    Ok(fsm.metadata)
}

#[inline]
fn peek_while<T, F>(iter: &mut std::iter::Peekable<T>, mut predicate: F)
    where T: Iterator,
          F: FnMut(&T::Item) -> bool
{
    while let Some(item) = iter.peek() {
        if !predicate(item) {
            break;
        }
        iter.next();
    }

}

#[inline]
fn next_until<T, F>(iter: &mut T, mut predicate: F)
    where T: Iterator,
          F: FnMut(T::Item) -> bool
{
    while let Some(item) = iter.next() {
        if predicate(item) {
            break;
        }
    }

}

fn render_head_variant(
    head: &str,
    permutation: &[usize]
) -> Result<Hotkey, String> {
    let mut walker = DelimSplit::new(head, 1, split_separator).peekable();
    let mut set_index = 0;


    let mut modifiers = 0;
    let mut key = None;
    let mut chords = Vec::new();

    fn push_chord<'a>(
        chords: &mut Vec<Chord>,
        key: &mut Option<Key>,
        modifiers: &mut Modifiers
    ) -> Result<(), StepError> {
        if let Some(code) = replace(key, None) {
            chords.push(Chord {
                key: code,
                modifiers: replace(modifiers, 0),
            });
            Ok(())
        } else {
            Err("No key set".into())
        }
    };

    while let Some((field, _, _row)) = walker.next() {
        match field {
            "{{" => {
                let mut count = 0;
                let choice = permutation[set_index];
                peek_while(&mut walker, |(peek, _, _)| {
                    if count >= choice {
                        false
                    } else {
                        if *peek == "," {
                            count += 1;
                        }
                        true
                    }
                });
            }
            // 'first_pass()' ensures ',' is never outside of '{{..}}'
            "," => next_until(&mut walker, |(field,_,_)| field == "}}"),
            "}}" => set_index += 1,
            ";" => push_chord(&mut chords, &mut key, &mut modifiers)?,

            "shift" => modifiers |= Mod::Shift as Modifiers,
            "super" => modifiers |= Mod::Super as Modifiers,
            "ctrl" => modifiers |= Mod::Ctrl as Modifiers,
            "alt" => modifiers |= Mod::Alt as Modifiers,

            _ if key.is_some() => panic!("Key already defined"),
            _  => {
                if let Some(i) = KEYSTRS.iter().position(|x| *x == field) {
                    key = Some(KEYCODES[i].clone());
                } else {
                    return Err(format!("Key {:?} not found", field));
                }
            }
        }
    }
    push_chord(&mut chords, &mut key, &mut modifiers)?;
    Ok(Hotkey(chords))
}


fn split_separator(substr: &str) -> Range<usize> {
    let mut chars = substr.chars();
    let mut delim_start = 0;
    let mut delim_close = 0;

    while let Some(ch) = chars.next() {
        delim_close += ch.len_utf8(); // represents post index
        // At this point, `ch == &substr[delim_start..delim_close]`
        match ch {
            '{' | '}' if delim_start == 0 => {
                chars.next();
                delim_close += '}'.len_utf8();
                delim_start = delim_close;
                break;
            }
            '{' | '}' => return delim_start..delim_start,
            ',' | ';' if delim_start == 0 => {
                delim_start = delim_close;
                break;
            }
            ',' | ';' => return delim_start..delim_start,
            _ if SEPARATOR.contains(&ch) => break,
            _ => delim_start = delim_close, // represents prev index
        }
    }

    // Eat separators
    while let Some(ch) = chars.next() {
        match ch {
            _ if !SEPARATOR.contains(&ch) => break,
            _ => {}
        }
        // Although this is a post-index, add after to simulate 'chars.peek()'
        delim_close += ch.len_utf8(); // Post last separator
    }
    delim_start..delim_close
}

//fn third_pass(metadata: EntryBlobMetadata) {
//}

//impl<'a> IntoIterator for EntryBlobMetadata<'a> {
//    type Item = (&'a mut[&'a str], &'a mut[Cow<'a, str>]);
//    type IntoIter = VariantGenerator<'a>;
//}
struct PermutationsGenerator<'a> {
    //entries: Vec<
    head_calculator_memory: Vec<usize>,
    body_calculator_memory: Vec<usize>,
    head_variant_memory: Vec<&'a str>,

    //head_variants: Vec<&'a str>,
    //body_variants: Vec<Cow<'a, str>>, // Dealing with escaping with owned data

    entries: Vec<UnparsedEntry<'a>>,
}
impl<'a> EntryBlobMetadata<'a> {
    fn allocate(self) -> PermutationsGenerator<'a> {
        PermutationsGenerator {
            head_calculator_memory: vec![0; self.max_head_set_count * 3],
            body_calculator_memory: vec![0; self.max_body_set_count * 3],
            head_variant_memory: Vec::with_capacity(self.max_permutation_count),

            //head_variants: Vec::with_capacity(head_len),
            //body_variants: Vec::with_capacity(body_len),

            entries: self.entries,
        }
    }
}

#[derive(Debug)]
struct Calculator<'b> {
    permutation: &'b mut [usize],
    set_sizes: &'b mut [usize],
    digit_values: &'b mut [usize],
}
impl<'b> Calculator<'b> {
    fn new(
        source: &str,
        set_count: usize,
        memory: &'b mut [usize],
    ) -> Self {
        let (permutation, rest) = memory.split_at_mut(set_count);
        let (set_sizes, rest) = rest.split_at_mut(set_count);
        let (digit_values, _) = rest.split_at_mut(set_count);
        if set_count > 0 { // split of non-blank is minimum 'len()' 1
            // Splits into regular keys and optional (enumerated) keys
            let reg_opt_pairs = DelimSplit::new(source, 1, split_brackets);
            for (i, (_, brackets, _)) in reg_opt_pairs.enumerate() {
                if !brackets.is_empty() { // Must be at least "{{}}"
                    set_sizes[i] = brackets.split(',').count();
                }
            }
            let mut product = 1;
            for (i, total) in set_sizes.iter().enumerate().rev() {
                digit_values[i] = product;
                product *= *total;
            }
        }
        Calculator {
            permutation,
            set_sizes,
            digit_values,
        }
    }

    fn permute(&mut self, permutation_index: usize) -> &[usize] {
        for i in 0..self.permutation.len() {
            let x = permutation_index / self.digit_values[i];
            self.permutation[i] = x % self.set_sizes[i];
        }
        &self.permutation
    }
}


//impl<'a> EntryBlobMetadata<'a> {
//    fn allocate(self) -> PermutationsGenerator<'a> {
//        let memory_mid = self.max_head_set_count * 2;
//        let memory_total = memory_mid + self.max_body_set_count * 2;
//        let (head_len, body_len) = self.entries.iter()
//            .fold((0, 0),|(mut head_len, mut body_len), entry| {
//                head_len += if entry.head_set_count == 0 {
//                    1
//                } else {
//                    entry.head_set_count * entry.permutation_count + 2
//                };
//                body_len += if entry.body_set_count == 0 {
//                    1
//                } else {
//                    entry.body_set_count * entry.permutation_count + 2
//                };
//                (head_len, body_len)
//            });
//        PermutationsGenerator {
//            calculator_memory: Vec::with_capacity(memory_total),
//            memory_mid,
//            head_variants: Vec::with_capacity(head_len),
//            body_variants: Vec::with_capacity(body_len),
//
//            entries: self.entries,
//        }
//    }
//}

//struct EntryBlobMetadata<'a> {
//    entries: Vec<UnparsedEntry<'a>>,
//    max_head_set_count: usize,
//    max_body_set_count: usize,
//}
//struct UnparsedEntry<'a> {
//    head: &'a str,
//    body: &'a str,
//    head_set_count: usize,
//    body_set_count: usize,
//    permutation_count: usize,
//    row: usize,
//}



//struct EntryVariantGenerator<'a> {
//    head: &'a str,
//    head_set_sizes: &'a [usize],
//    head_permutation: &'a [usize],
//    head_memory: &'a [&'a str],
//
//    body: &'a str,
//    body_set_sizes: &'a [usize],
//    body_permutation: &'a [usize],
//    body_memory: &'a [Cow<'a,str>],
//
//    index: usize,
//    permutation_count: usize,
//}
//
//impl<'a> Iterator for EntryVariantGenerator<'a> {
//    type Item = Variant<'a>;
//    fn next(&mut self) -> Option<Self::Item> {
//        None
//    }
//}
//struct Variant<'a> {
//    head: &'a str,
//    body: &'a [Cow<'a, str>],
//}

#[test]
fn parser() {
    //let line = r#"super {{space, x}} ; super {{w,y,z}} ; super {{a,b,c,d}}| $TERMINAL -e sh -c 'echo "nmcli"; echo "===="; sudo nmtui'; statusbar-startrefresh.sh"#;
    //println!("line: {:?}", line);
    //parse_entry(line);

    let _line = r#"|super {{x, y}} ; super {{a }} ; super {{a,b}}|
        echo {{1,2,3,4}}
    "#;
    //println!("{:#?}", parse_entry(line));
    //println!("{:?}", split_head_body_and_validate(&line['|'.len_utf8()..]));
    //println!("{:?}", &line[line.len()..]);

    let _file = r#"
    #
#hello
|super {{, alt, ctrl, ctrl alt}} Return|
  {{$TERMINAL, alacritty, st, sakura}} -e tmux.sh open
|super {{c, t,g}} ; super {{b,s}}|
  $TERMINAL -e {{curl,browser.sh}}  '{{terminal,gui}}' '{{bookmarks,search}}'

|super shift q|"#;
    let first = first_pass(_file).unwrap();
    if false {
        first.entries.iter().for_each(|x| {
            println!(
                "{}|==={}=== {} {} {}",
                x.row, x.head, x.head_set_count, x.body_set_count, x.permutation_count
            );
            println!("{:?}", x.body);
        });
        println!("Head sets: {}", first.max_head_set_count);
        println!("Body sets: {}", first.max_body_set_count);
        //println!("Second {:?}", first);
    }

    let mut second = first.allocate();
    for UnparsedEntry {
        row: _row,
        head,
        head_set_count,
        body,
        body_set_count,
        permutation_count,
    } in second.entries {
        let mut head_calc = Calculator::new(
            head,
            head_set_count,
            &mut second.head_calculator_memory,
        );
        let mut body_calc = Calculator::new(
            body,
            body_set_count,
            &mut second.body_calculator_memory,
        );
        //let head_variant = &mut second.head_variant_memory;
        //head_variant.clear();
        for i in 0..permutation_count {
            let hotkey = render_head_variant(head, head_calc.permute(i));
            //let cmd = render_body(body, permutation_count, body_calc.permute(i));
            let cmd = render_body(body.trim(), permutation_count, body_calc.permute(i));

            println!("{}", hotkey.unwrap());
            println!("  {}", cmd.join(""));
        }
    }
}

use std::borrow::Cow;
fn render_body<'a>(
    body: &'a str,
    set_count: usize,
    permutation: &[usize]
) -> Vec<Cow<'a, str>> {
    let mut output = Vec::with_capacity(set_count);
    let mut buffer = String::new();
    let split = DelimSplit::new(body, 1, split_brackets);
    for (set_index, (regular, delim, _row)) in split.enumerate() {
        output.push(regular.into());

        buffer.clear();
        let delim = if delim.is_empty() {
            delim
        } else {
            buffer.reserve(delim.len() - "{{}}".len());
            &delim["{{".len()..]
        };

        // Basically a `delim.split(',')` but with escaping backslash
        // Additionally escaped newlines are ignored (similar to shellscript)
        // Push the delim when we get to the correct field
        let mut walker = delim.chars().peekable();
        let mut start = 0;
        let mut until = start;
        let mut field_index = 0;
        while let Some(ch) = walker.next() {
            match ch {
                '\\' => {
                    buffer.push_str(&delim[start..until]);
                    let escaped = walker.next().unwrap();
                    if escaped != '\n' {
                        buffer.push(escaped); // Special case escaped newline
                    }
                    until += '\\'.len_utf8() + escaped.len_utf8();
                    start = until;
                }
                ',' | '}' => {
                    if field_index == permutation[set_index] {
                        buffer.push_str(&delim[start..until]);
                        output.push(buffer.split_off(0).into());
                        break;
                    }
                    debug_assert_eq!(','.len_utf8(), '}'.len_utf8());
                    start = until + ','.len_utf8();
                    until = start;
                    field_index += 1;
                    buffer.clear();
                }
                c => until += c.len_utf8(),
            }
        }
        //println!("{:?} {:?}", regular, brackets);
    }
    output
}




/******************************************************************************
 * A 'std::str::Chars' wrapper for use in 'first_pass()'
 ******************************************************************************/
struct CharsWithIndex<'a> {
    pub(self) iter: std::iter::Peekable<std::str::Chars<'a>>,
    prev: usize,
    post: usize,
    row: usize,
    col: usize,
    last_char: char,
}
impl<'a> CharsWithIndex<'a> {
    fn new(text: &'a str, start_row: usize) -> Self {
        let last_char = ' ';
        debug_assert!(last_char != '\n');
        Self {
            iter: text.chars().peekable(),
            prev: 0,
            post: 0,
            row: start_row,
            col: 0,
            last_char,
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        self.iter.peek()
    }

    fn eat_whitespace(&mut self) {
        while let Some(peek) = self.iter.peek() {
            if peek.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn eat_separator(&mut self) {
        while let Some(peek) = self.iter.peek() {
            if SEPARATOR.contains(peek) {
                self.next();
            } else {
                break;
            }
        }
    }
}

//
impl<'a> Iterator for CharsWithIndex<'a> {
    type Item = char;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(c) = self.iter.next() {
            // This is sound in the first '.next()' case
            // (prev, post) => (0, 0).next() -> (0, 1)
            self.prev = self.post;
            self.post += c.len_utf8();

            self.col += 1;
            if self.last_char == '\n' {
                self.row += 1;
                self.col = 1;
            }
            self.last_char = c;

            Some(c)
        } else {
            self.prev = self.post;
            None
        }
    }
}

#[test]
fn chars_with_index() {
    let mut iter = CharsWithIndex::new("a", 1);
    assert_eq!(iter.next(), Some('a'));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let mut iter = CharsWithIndex::new("", 1);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let mut iter = CharsWithIndex::new("你m好!!我是y只mao", 1);
    assert_eq!(iter.next(), Some('你'));
    assert_eq!(iter.next(), Some('m'));
    assert_eq!(iter.next(), Some('好'));
    assert_eq!(iter.next(), Some('!'));
    assert_eq!(iter.next(), Some('!'));
    assert_eq!(iter.next(), Some('我'));
    assert_eq!(iter.next(), Some('是'));
    assert_eq!(iter.next(), Some('y'));
    assert_eq!(iter.next(), Some('只'));
    assert_eq!(iter.next(), Some('m'));
    assert_eq!(iter.next(), Some('a'));
    assert_eq!(iter.next(), Some('o'));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let source = "你m好!!我是y只mao";
    let mut iter = CharsWithIndex::new(source, 1);
    while let Some(c) = iter.next() {
        assert_eq!(&c.to_string(), &source[iter.prev..iter.post]);
    }

    // TODO: test peek and eat_whitespace
    //let mut iter = CharsWithIndex::new("你m好!!我", 1);
}

/******************************************************************************
 * A 'std::str::Chars' wrapper for use in 'first_pass()'
 ******************************************************************************/
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

fn split(source: &str, start_row: usize) -> DelimSplit {
    DelimSplit {
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
        },
    }
}

struct DelimSplit<'a> {
    buffer: &'a str,
    row: usize,
    delimit_by: fn(&str) -> Range<usize>,
}
impl<'a> DelimSplit<'a> {
    fn new(s: &'a str, start_row: usize, f: fn(&str) -> Range<usize>) -> Self {
        Self {
            buffer: s,
            row: start_row,
            delimit_by: f,
        }
    }
}

impl<'a> Iterator for DelimSplit<'a> {
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
