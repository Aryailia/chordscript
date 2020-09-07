use std::borrow::Cow;
use crate::constants::{Chord, Hotkey, Shortcut};
use crate::parser::PermutationsGenerator;

// TODO: Remove this clone, maybe. Was used for prototyping
#[derive(Clone, Debug)]
enum KeyspaceAction<'a, 'b> {
    SetState(&'b [Chord]),
    Action(&'b [Cow<'a, str>]),
}

// Convert from a list of Shorcuts to a list of states
#[derive(Debug)]
pub struct Keyspace<'a, 'b> {
    activator: Hotkey<'b>,
    list: Vec<(Chord, KeyspaceAction<'a, 'b>)>,
}

impl<'a, 'b> Keyspace<'a, 'b> {
    fn push_shortcut(&mut self, shortcut: &Shortcut<'a, 'b>, chord_index: usize) {
        let chord_list = shortcut.hotkey.0;
        if chord_index + 1 == chord_list.len() {
            self.list.push((
                chord_list[chord_index].clone(),
                KeyspaceAction::Action(shortcut.action),
            ));
        } else if chord_index < chord_list.len() {
            self.list.push((
                chord_list[chord_index].clone(),
                KeyspaceAction::SetState(&chord_list[0..=chord_index]),
            ));
        } else {
            panic!("DEV");
        }
    }

    fn push_partition(&mut self, partition: ShortcutPartition<'a, 'b>, chord_index: usize) {
        if partition.len() >= 2 {
            let chord_list = &partition[0].hotkey.0;
            //println!("===\n{:?}\n{:?} {}\n", partition, chord_list, chord_index);
            self.list.push((
                chord_list[chord_index].clone(),
                KeyspaceAction::SetState(&chord_list[0..chord_index + 1]),
            ));
            //keyspace.push_shortcut(&partition[0], 0);
        } else {
            debug_assert!(partition.len() > 0 );
            self.push_shortcut(&partition[0], chord_index);
        }
    }

    fn print(&self) {
        self.list.iter().for_each(|entry| {
            println!("{}", entry.0);
            match entry.1 {
                KeyspaceAction::SetState(chord_list) =>
                    println!("  Set: {}", Hotkey(chord_list)),
                KeyspaceAction::Action(action) =>
                    println!("  Action: {:?}", action.join("")),
            }
        });
    }
}

// As a 'trait' rather than direct 'impl' so we import explicitly
pub trait KeyspaceList {
    fn keyspace_list(&self) -> Result<(), String>;
}
pub trait KeyspaceList2<'a> {
    fn keyspace_list<'b>(&'b self) -> Result<Vec<Keyspace<'a, 'b>>, String>;
}


// Most shortcut terminal
//impl<'a> KeyspaceList<'a> for PermutationsGenerator<'a> {
impl<'a> KeyspaceList for PermutationsGenerator<'a> {
    //fn keyspace_list<'b>(&'b self) -> Result<Vec<Keyspace<'a, 'b>>, String> {
    fn keyspace_list(&self) -> Result<(), String> {
        //use crate::constants::{Key, Mod};
        let shortcut_list = self.allocate_shortcut_list()?;

        let mut keyspace_list = Vec::new();
        let mut visited = VisitedTracker::new(shortcut_list.as_slice());
        shortcut_list.iter().enumerate().for_each(|(i, shortcut)| {
            let head_chord_list = shortcut.hotkey.0;
            for j in visited.last_unvisited_index(i)..head_chord_list.len() {
                // Build the keyspace
                let mut keyspace = Keyspace {
                    activator: Hotkey(&head_chord_list[0..j]),
                    list: vec![],
                };
                println!("{} {} {}", i, j, Hotkey(&head_chord_list[0..j]));
                visited.refine_partition_of_i_by_chord_j(i, j).for_each(|partition| {
                    //if i == 1 && j == 2 {
                    //    println!("{:?}", visited.tracker);
                    //}
                    //println!("  {}", partition.len());
                    keyspace.push_partition(partition, j);
                });
                keyspace_list.push(keyspace);
            }
        });
        println!("\n---\n---\n");
        //print_keyspace_list(&keyspace_list);
        //print_keyspace_list2(&keyspace_list);
        println!("{}", keyspace_list.len());
        Ok(())
        //Ok(keyspace_list)
    }
}

fn print_keyspace_list<'a, 'b>(keyspace_list: &[Keyspace<'a, 'b>]) {
    keyspace_list.iter().for_each(|keyspace| {
        println!("==== {} ===", keyspace.activator);
        keyspace.list.iter().for_each(|(chord, action)| {
            match action {
                KeyspaceAction::SetState(chord_list) =>
                    println!("{}: set {}", chord, Hotkey(chord_list)),
                KeyspaceAction::Action(a) =>
                    println!("{}: {:?}", chord, a.join("")),
            }
        });
    });
}

fn print_keyspace_list2<'a, 'b>(list: &[Keyspace<'a, 'b>]) {
    list.iter().for_each(|keyspace| {
        println!("==== {} ===", keyspace.activator);
        keyspace.print();
    });
}

fn print_shortcut_list<'a, 'b>(list: &[Shortcut<'a, 'b>]) {
    list.iter().for_each(|shortcut| {
        print!("  {}: ", shortcut.hotkey);
        println!("  {}", shortcut.action.join(""));
    });
}

struct VisitedTracker<'a, 'b> {
    shortcut_list: &'b [Shortcut<'a, 'b>],
    // (visited_chord_index, container_partition_len)
    tracker: Vec<(usize, usize)>,
}

impl<'a, 'b> VisitedTracker<'a, 'b> {
    fn new(shortcut_list: &'b [Shortcut<'a, 'b>]) -> Self {
        let len = shortcut_list.len();
        Self {
            shortcut_list,
            tracker: vec![(0, len); len],
        }
    }
    fn last_unvisited_index(&self, index: usize) -> usize {
        self.tracker[index].0
    }

    // Begins with whole 'self.shortcut_list'
    fn refine_partition_of_i_by_chord_j(
        &mut self,
        mut shortcut_index: usize,
        chord_index: usize,
    ) -> SharedChordWalker<'a, 'b> {
        let end = self.tracker[shortcut_index].1;

        let container_partition = &self.shortcut_list[shortcut_index..end];
        debug_assert!(container_partition.len() >= 1);
        debug_assert!(chord_index < container_partition[0].hotkey.0.len());
        SharedChordWalker {
            list: container_partition,
            chord_index,
        }.for_each(|partition| {
            let partition_start = shortcut_index;
            let partition_close = partition_start + partition.len();
            let visited = (chord_index + 1, partition_close);
            for i in 0..partition.len() {
                self.tracker[partition_start + i] = visited;
            }
            shortcut_index += partition.len();

        });

        SharedChordWalker {
            list: container_partition,
            chord_index,
        }
    }
}

type ShortcutPartition<'a, 'b> = &'b [Shortcut<'a, 'b>];

fn split_by_shared_chord<'a, 'b>(list: &'b [Shortcut<'a, 'b>], chord_index: usize) -> SharedChordWalker<'a ,'b> {
    debug_assert!(list.len() >= 1);
    debug_assert!(chord_index < list[0].hotkey.0.len());

    SharedChordWalker {
        list,
        chord_index,
    }
}

struct SharedChordWalker<'a, 'b> {
    list: &'b [Shortcut<'a,'b>],
    chord_index: usize,
}

impl<'a, 'b> Iterator for SharedChordWalker<'a, 'b> {
    type Item = ShortcutPartition<'a, 'b>;
    fn next(&mut self) -> Option::<Self::Item> {
        if self.list.len() > 0 {
            let y = self.chord_index;
            let first_hotkey = self.list[0].hotkey.0;
            if first_hotkey.len() > self.chord_index {
                let mut close = 1;
                for shortcut in &self.list[1..] {
                    let hotkey = shortcut.hotkey.0;
                    if y >= hotkey.len() || &hotkey[y] != &first_hotkey[y] {
                        break
                    }
                    close += 1;
                }
                let partition = &self.list[0..close];
                self.list = &self.list[close..];
                Some(partition)
            } else {
                None
            }
        } else {
            None
        }
    }
}

