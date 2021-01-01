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
mod keyspace;
mod parser;

//use std::fs::File;
//use std::io::{BufRead, BufReader};
use keyspace::{Keyspace, KeyspaceList};
use parser::{parse_into_shortcut_list, validate_and_calculate_allocations};

// This is per entry
const PERMUTATION_LIMIT: usize = 1000;

enum ListType {
    Shortcuts,
    Keyspaces,
}

fn display_help(msg: String) -> ! {
    eprintln!("Help {}", msg);
    exit(1)
}

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

fn main() {
    let (config, output_type, command_builder) = parse_args();
    let file = match fs::read_to_string(&config) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error reading file: {:?}\n{}", config.display(), err);
            exit(1)
        }
    };
    let metadata = validate_and_calculate_allocations(file.as_str()).or_die(1);
    let parser_storage = parse_into_shortcut_list(metadata).or_die(1);
    match output_type {
        ListType::Shortcuts => {
            let list = parser_storage.allocate_shortcut_list().or_die(1);
            let mut command = command_builder.instantiate();
            list.iter().for_each(|shortcut| {
                command.arg(format!("{}", shortcut.hotkey));
                command.arg(format!("{}", shortcut.action.join("")));
            });
            command_builder.run_and_exit_on_error(command);
        }
        ListType::Keyspaces => {
            let list = parser_storage.allocate_keyspace_list().or_die(1);
            list.iter().for_each(|keyspace| {
                let (title, chord_action_bi_list) = format_keyspace(keyspace);
                let mut command = command_builder.instantiate();
                command.arg(title);
                command.args(chord_action_bi_list);
                command_builder.run_and_exit_on_error(command);
            });
        }
    };

    //run: cargo build; time cargo run config.txt keyspace-list ./keyspace-list.sh api
}

use keyspace::KeyspaceAction;

fn format_keyspace(keyspace: &Keyspace) -> (String, Vec<String>) {
    let mut chord_actions = Vec::with_capacity(keyspace.list.len() * 3);
    keyspace.list.iter().for_each(|(chord, action)| {
        chord_actions.push(format!("{}", chord));
        match action {
            KeyspaceAction::SetState(chord_list) => {
                chord_actions.push("state".into());
                chord_actions.push(format!("{}", constants::Hotkey(chord_list)));
            }
            KeyspaceAction::Action(action_cow_list) => {
                chord_actions.push("run".into());
                chord_actions.push(action_cow_list.join(""));

            }
        }
    });
    (format!("{}", keyspace.title), chord_actions)
}


struct ClonableCommand {
    process: String,
    args: Vec<String>,
}

impl ClonableCommand {
    fn instantiate(&self) -> Command {
        let mut command = Command::new(self.process.as_str());
        command.args(&self.args);
        command
    }

    fn run_and_exit_on_error(&self, mut command: Command) -> i32 {
        match command.status() {
            Ok(status) if status.success() => 0,
            Ok(status) => {
                let code = status.code().unwrap_or(0);
                eprintln!("{:?}: exited with code {}", self.process, code);
                exit(code)
            }
            Err(err) => {
                eprintln!("{:?}: {}", self.process, err);
                exit(1)
            }
        }
    }
}


fn parse_args() -> (PathBuf, ListType, ClonableCommand) {
    let mut args_iter = env::args();
    args_iter.next(); // skip $0
    let config = match args_iter.next() {
        Some(s) => PathBuf::from(s),
        _ => display_help("No config file to parse".into()),
    };
    error_if_file_missing(config.as_path());

    let output_type = match args_iter.next() {
        Some(s) if s.as_str() == "shortcut-list" => ListType::Shortcuts,
        Some(s) if s.as_str() == "keyspace-list" => ListType::Keyspaces,
        Some(s) => display_help(format!("{:?} is an invalid choice.", s)),
        _ => display_help("No list format specified".into()),
    };

    let command_string = match args_iter.next() {
        Some(s) => s,
        _ => display_help("No command run".into()),
    };
    (config, output_type, ClonableCommand {
        process: command_string,
        args: args_iter.collect::<Vec<String>>(),
    })
}

fn error_if_file_missing(path: &Path) {
    if !path.is_file() {
        eprintln!(
            "Cannot find config file {:?}\n{}",
            path.display(),
            match path.canonicalize().or(PathBuf::from(".").canonicalize()) {
                Ok(p) => format!("Full path targetted: {:?}", p.display()),
                Err(err) => format!("{}", err),
            }
        );
        exit(1);
    }
}

trait PrintError<T> {
    fn or_die(self, exit_code: i32) -> T;
}
impl<T, E: std::fmt::Display> PrintError<T> for Result<T, E> {
    fn or_die(self, exit_code: i32) -> T {
        match self {
            Ok(x) => x,
            Err(err) => {
                eprintln!("{}", err);
                exit(exit_code)
            }
        }
    }
}

#[test]
fn asdf() {
    let _file = r#"
    #
#hello
|super {{, alt, ctrl, ctrl alt}} Return|
  {{$TERMINAL, alacritty, st, sakura}} -e tmux.sh open
|super {{c, t,g, k, v}} ; super {{b,s}}|
  $TERMINAL -e {{curl,browser.sh}}  '{{terminal,gui}}' '{{bookmarks,search}}'

|super shift q|"#;

    let _file = r#"
#
#|super {{a,b}};super {{d,e,f}};super{{g,h}}|
|super a; super b| echo 2
|super a| echo 1

"#;
    let _file = r#"
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
}
