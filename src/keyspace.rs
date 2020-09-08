use crate::constants::{Chord, Hotkey, Shortcut};
use crate::parser::PermutationsGenerator;
use std::borrow::Cow;

// TODO: Remove this clone, maybe. Was used for prototyping
#[derive(Clone, Debug)]
enum KeyspaceAction<'a, 'b> {
    SetState(&'b [Chord]),
    Action(&'b [Cow<'a, str>]),
}

// Convert from a list of Shorcuts to a list of states
#[derive(Debug)]
pub struct Keyspace<'a, 'b> {
    // The path to this keyspace (does not do anything semantically)
    title: Hotkey<'b>,
    // The list of keys. While inside this keyspace, press 'Chord' to
    // performan 'KeyspaceAction'
    list: Vec<(Chord, KeyspaceAction<'a, 'b>)>,
}

// As a 'trait' rather than direct 'impl' so we import explicitly
pub trait KeyspaceList<'a> {
    fn allocate_keyspace_list<'b>(&'b self) -> Result<Vec<Keyspace<'a, 'b>>, String>;
}

// This a means of providing window managers that do not support chaining
// keyboard chords the means to do so (if there is a way to set state).
//
// Abstractly, this is takes the shortcut list and parses it into a tree where
// the non-leaf nodes are single chords (where a path from root to leaf is
// the shortcut hotkey) and the leaf nodes are the associated shortcut actions.
//
// Then flattens this tree into an associative array of Keyspace, which is
// conceptually: state + Chord => action. A 'Keyspace' is essentially any
// edge between two nodes in the tree.
//
// In implementation, we do not use a tree. Instead, we make use of
// 'VisitedTracker' which is just a (usize, usize) of the same length as
// the shortcut_list to track the traversal of the shortcut list
impl<'a> KeyspaceList<'a> for PermutationsGenerator<'a> {
    fn allocate_keyspace_list<'b>(&'b self) -> Result<Vec<Keyspace<'a, 'b>>, String> {
        let shortcut_list = self.allocate_shortcut_list()?;
        let mut keyspace_list = Vec::new();
        let mut visited = VisitedTracker::new(shortcut_list.as_slice());
        for (i, shortcut) in shortcut_list.iter().enumerate() {
            let first_hotkey_chord_list = shortcut.hotkey.0;

            // Loops through all chords in 'shortcut' and creating
            // 1. a 'Keyspace' for every k but the last chord index
            //    So [Ctrl + A; Ctrl + B; Ctrl + C] make two 'Keyspace''s
            // 2. a new 'KeyspaceAction' for every pair (k, k + 1) for all k
            // 3. Through 'visited', ensure we do not double process entries
            //
            // Continually narrow the containing partition by chord,
            for j in visited.unvisited_chord_indicies(i) {
                // Build the keyspace
                let mut space = Keyspace {
                    title: Hotkey(&first_hotkey_chord_list[0..j]),
                    list: vec![],
                };
                visited
                    // Updates 'visited' so we know what has been processed
                    // and also provides an iterator
                    .refine_container_for_i_partitioning_by_chord_j(i, j)
                    // Make the appropriate 'KeyspaceAction'
                    .for_each(|partition| space.push_partition(partition, j));
                keyspace_list.push(space);
            }
        }
        Ok(keyspace_list)
    }
}

impl<'a, 'b> Keyspace<'a, 'b> {
    fn push_partition(&mut self, partition: &[Shortcut<'a, 'b>], chord_index: usize) {
        if partition.len() >= 2 {
            let chord_list = &partition[0].hotkey.0;
            //println!("===\n{:?}\n{:?} {}\n", partition, chord_list, chord_index);
            self.list.push((
                chord_list[chord_index].clone(),
                KeyspaceAction::SetState(&chord_list[0..chord_index + 1]),
            ));
        //keyspace.push_shortcut(&partition[0], 0);
        } else {
            debug_assert!(partition.len() > 0);
            self.push_shortcut(&partition[0], chord_index);
        }
    }

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

    pub fn print(&self) {
        if self.title.0.is_empty() {
            println!("=== '' ===");
        } else {
            println!("=== {} ===", self.title);
        }
        // TODO: making alignment work (fix chord Display trait)
        self.list.iter().for_each(|entry| match entry.1 {
            KeyspaceAction::SetState(chord_list) => {
                println!("{0: <40} | Set: {1}", entry.0, Hotkey(chord_list))
            }
            KeyspaceAction::Action(action) => {
                println!("{0: <40} | Action: {1}", entry.0, action.join(""))
            }
        });
    }
}

pub fn print_shortcut_list<'a, 'b>(list: &[Shortcut<'a, 'b>]) {
    list.iter().for_each(|shortcut| {
        print!("  {}: ", shortcut.hotkey);
        println!("  {}", shortcut.action.join(""));
    });
}

struct VisitedTracker<'a, 'b, 'c> {
    shortcut_list: &'c [Shortcut<'a, 'b>],

    // .0: index in 'shortcut_list[?].hotkey.0' after chords already processed
    // .1: container_paritition_end (pos in 'shortcut_list')
    tracker: Vec<(usize, usize)>,
}

impl<'a, 'b, 'c> VisitedTracker<'a, 'b, 'c> {
    fn new(shortcut_list: &'c [Shortcut<'a, 'b>]) -> Self {
        let len = shortcut_list.len();
        Self {
            shortcut_list,
            tracker: vec![(0, len); len],
        }
    }

    fn unvisited_chord_indicies(&self, index: usize) -> std::ops::Range<usize> {
        let last_visited = self.tracker[index].0;
        let total_chords = self.shortcut_list[index].hotkey.0.len();
        last_visited..total_chords
    }

    //// Begins with whole 'self.shortcut_list'
    fn refine_container_for_i_partitioning_by_chord_j(
        &mut self,
        shortcut_index: usize, // i
        chord_index: usize,    // j
    ) -> SharedChordPartitioner<'a, 'b, 'c> {
        let close = self.tracker[shortcut_index].1;
        let container_partition = &self.shortcut_list[shortcut_index..close];

        let chunk_container_again_by_chord_j = SharedChordPartitioner {
            list: container_partition,
            chord_index,
        };

        debug_assert!(container_partition.len() >= 1);
        debug_assert!(chord_index < container_partition[0].hotkey.0.len());

        let mut index = shortcut_index;
        chunk_container_again_by_chord_j
            .clone()
            .for_each(|partition| {
                let subpartition_len = partition.len();
                let subpartition_close = index + subpartition_len;
                for _ in 0..subpartition_len {
                    self.tracker[index] = (chord_index + 1, subpartition_close);
                    index += 1;
                }
            });

        chunk_container_again_by_chord_j
    }
}

#[derive(Clone)]
struct SharedChordPartitioner<'a, 'b, 'c> {
    list: &'c [Shortcut<'a, 'b>],
    chord_index: usize,
}

impl<'a, 'b, 'c> Iterator for SharedChordPartitioner<'a, 'b, 'c> {
    type Item = &'c [Shortcut<'a, 'b>];
    fn next(&mut self) -> Option<Self::Item> {
        if self.list.len() > 0 {
            let y = self.chord_index;
            let first_hotkey = self.list[0].hotkey.0;
            if first_hotkey.len() > self.chord_index {
                let mut close = 1;
                for shortcut in &self.list[1..] {
                    let hotkey = shortcut.hotkey.0;
                    if y >= hotkey.len() || &hotkey[y] != &first_hotkey[y] {
                        break;
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
