//run: cargo build; time cargo run 2>/dev/null

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

#![allow(dead_code)]


mod constants;
mod parser;

//use std::fs::File;
//use std::io::{BufRead, BufReader};
use parser::{validate_and_calculate_allocations, parse_into_shortcut_list};

const PERMUTATION_LIMIT: usize = 1000;



fn main() {
    //let file = File::open("main.rs").unwrap();
    //let reader = BufReader::new(file);
    //for (index, line) in reader.lines().enumerate() {
    //    let line = line.unwrap();
    //    println!("{}  {}", index + 1, line);
    //}

    //let _line = r#"|super {{x, y}} ; super {{a }} ; super {{a,b}}|
    //    echo {{1,2,3,4}}
    //"#;

    let _file = r#"
    #
#hello
|super {{, alt, ctrl, ctrl alt}} Return|
  {{$TERMINAL, alacritty, st, sakura}} -e tmux.sh open
|super {{c, t,g, k, v}} ; super {{b,s}}|
  $TERMINAL -e {{curl,browser.sh}}  '{{terminal,gui}}' '{{bookmarks,search}}'

|super shift q|"#;
    let first_pass = validate_and_calculate_allocations(_file).unwrap();
    let mut list_owner = parse_into_shortcut_list(first_pass).unwrap();
    let mut shortcut_list = list_owner.shortcut_list();

    if true {
        println!("========\n{}\n========", _file);
        while let Some(shortcut) = shortcut_list.next() {
            println!("> {}", shortcut.hotkey);
            println!("  {}", shortcut.action.join(""));
        }
    }
    //let state_list = parse_into_keyspace_list(shortcut_list);

}

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


