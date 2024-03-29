use crate::constants::{KEYCODES, MODIFIERS};
use crate::deserialise::{Print, TrimEscapeStrList};
use crate::parser::keyspaces::{Action, KeyspaceOwner};
use crate::structs::{Chord, Shortcut};
use crate::{array, precalculate_capacity_and_build};
use super::DeserialisedChord;

pub struct KeyspacePreview<'keyspaces, 'parsemes, 'filestr>(
    pub &'keyspaces KeyspaceOwner<'parsemes, 'filestr>,
);
pub struct KeyspaceAction<'keyspaces, 'parsemes, 'filestr>(pub &'keyspaces Action<'parsemes, 'filestr>);

impl<'keyspaces, 'parsemes, 'filestr> Print for KeyspacePreview<'keyspaces, 'parsemes, 'filestr> {
    precalculate_capacity_and_build!(self, buffer {
        let mut iter = self.0.to_iter();
        let first = iter.next().unwrap();
        let mode = "\nmode \"";
        let mode_begin = "\" {\n";
        let mode_close = "}\n";
        let padding = "  ";
    } {
        array!(@len { first.actions } |> "", KeyspaceAction, "\n") =>
            array!(@push { first.actions } |> "", KeyspaceAction, "\n", |> buffer);

        iter.map(|ks|
            mode.len()
            //+ ks.title.string_len()
            + array!(@len_join { ks.title } |> wrap_chord, " ; ")
            + mode_begin.len()
            + array!(@len { ks.actions } |> padding, KeyspaceAction, "\n")
            + mode_close.len()
        ).sum::<usize>() => iter.for_each(|ks| {
            buffer.push_str(mode);
            array!(@push_join { ks.title } |> wrap_chord, " ; ", |> buffer);
            buffer.push_str(mode_begin);
            array!(@push { ks.actions } |> padding, KeyspaceAction, "\n", |> buffer);
            buffer.push_str(mode_close);
        });
    });
}

#[inline]
fn wrap_chord<'a, 'b>(chord: &'a Chord<'b>) -> DeserialisedChord<'a, 'b> {
    DeserialisedChord(" ", chord, &KEYCODES, &MODIFIERS)
}

impl<'keyspaces, 'parsemes, 'filestr> Print for KeyspaceAction<'keyspaces, 'parsemes, 'filestr> {
    precalculate_capacity_and_build!(self, buffer {
        let mode = "MODE: ";
        let arrow = " -> ";
        let quote = '\'';
        let candidates = &['\''];
        let escape = &["'\\''"];
        let hotkey_trigger = self.0.key_trigger();
    } {
        match &self.0 {
            Action::SetState(title) =>
                mode.len()
                + wrap_chord(hotkey_trigger).string_len()
                + arrow.len()
                //+ title.string_len(),
                + array!(@len_join { title } |> wrap_chord, " ; "),
            Action::Command(_, Shortcut { command, .. }) =>
                wrap_chord(hotkey_trigger).string_len()
                + arrow.len()
                + TrimEscapeStrList(quote, candidates, escape, command).string_len(),
        } => match &self.0 {
            Action::SetState(title) => {
                buffer.push_str(mode);
                wrap_chord(hotkey_trigger).push_string_into(buffer);
                buffer.push_str(arrow);
                //title.push_string_into(buffer);
                array!(@push_join { title } |> wrap_chord, " ; ", |> buffer);
            }
            Action::Command(_, Shortcut { command, .. }) => {
                wrap_chord(hotkey_trigger).push_string_into(buffer);
                buffer.push_str(arrow);
                TrimEscapeStrList(quote, candidates, escape, command).push_string_into(buffer);
            }
        };
    });
}
