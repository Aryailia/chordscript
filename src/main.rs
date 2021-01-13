//run: cargo test -- --nocapture

#![allow(dead_code)]
#![allow(clippy::string_lit_as_bytes)]

mod constants;
mod errors;
mod keyspace;
mod lexer;
mod macros;
mod parser;
mod reporter;
mod structs;

fn main() {
    println!("Hello, world!");
}

#[test]
fn interpret() {
    let _file = r#"
    #
#hello
|super {{, alt, ctrl, ctrl alt}} Return|
  {{$TERMINAL, alacritty, \
  st, sakura}} -e tmux.sh open
|super {{c, t, g, k}} ; super {{b,s}}|
  $TERMINAL -e {{curl,browser.sh}}  '{{terminal,gui}}' '{{bookmarks,search}}'
{{{| cat -}}}
#| | echo asdf  # @TODO this should not be a lexer error
#|super;| echo yo
#|| echo yo
#|super shift t|echo {{3349\, 109324}}
|super shift q|"#;
    //println!("{}", _file);

    if let Err(err) = (|| -> Result<(), reporter::MarkupError> {
        let _lexemes = lexer::process(_file)?;
        //_lexemes.to_iter().for_each(print_lexeme);
        //_lexemes.to_iter().for_each(debug_print_lexeme);

        let parsemes = parser::process(&_lexemes)?;
        //let mut _hotkeys = parsemes.make_owned_sorted_view();
        //_hotkeys.iter().for_each(|shortcut| println!("{}", shortcut));
        let _keyspaces = keyspace::process(&parsemes);
        keyspace::debug_print_keyspace_owner(&_keyspaces);
        Ok(())
    })() {
        println!("{}", err);
    }
}

fn debug_print_lexeme(lexeme: lexer::Lexeme) {
    let head = lexeme
        .head
        .iter()
        .map(|x| format!("{:?}", x))
        .collect::<Vec<_>>()
        .join(" ");
    let body = lexeme
        .body
        .iter()
        .map(|x| format!("{:?}", x))
        .collect::<Vec<_>>()
        .join(" ");
    print!("|{}|\n  {}\n\n", head, body);
}
fn print_lexeme(lexeme: lexer::Lexeme) {
    use constants::{KEYCODES, MODIFIERS};
    use lexer::BodyType;
    use lexer::HeadType;

    let head = lexeme
        .head
        .iter()
        .filter_map(|head_lexeme| match head_lexeme.data {
            HeadType::Mod(k) => Some(MODIFIERS[k]),
            HeadType::Key(k) => Some(KEYCODES[k]),
            HeadType::ChoiceBegin => Some("{{"),
            HeadType::ChoiceDelim => Some(","),
            HeadType::ChoiceClose => Some("}}"),
            HeadType::ChordDelim => Some(";"),
            HeadType::Blank => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    let body = lexeme
        .body
        .iter()
        .map(|body_lexeme| match body_lexeme.data {
            BodyType::Section => body_lexeme
                .as_str()
                .lines()
                .map(|line| format!("{:?}", line))
                .collect::<Vec<_>>()
                .join("\n  "),
            BodyType::ChoiceBegin => "\n  {{\n    ".to_string(),
            BodyType::ChoiceDelim => ",\n    ".to_string(),
            BodyType::ChoiceClose => ",\n  }}\n  ".to_string(),
        })
        .collect::<Vec<_>>()
        .join("");
    print!("|{}|\n  {}\n\n", head, body.trim());
}
