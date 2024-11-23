use crate::backend::errors::error_location::SourceOffset;
use crate::{
    backend::errors::{
        backend_error::BackendError, backend_error_kind::BackendErrorKind, diagnostic::Diagnostic,
        diagnostic_kind::DiagnosticKind, error_location::ErrorLocation,
    },
    parser::{RawTick, TabElement},
};

use super::{comment_line, partline, Measure, Partline, Section};
#[derive(Debug)]
pub struct Parse2Result {
    pub diagnostics: Vec<Diagnostic>,
    pub sections: Vec<Section>,

    pub string_names: [char; 6],
    pub strings: [Vec<RawTick>; 6],
    pub measures: [Vec<Measure>; 6],
    pub offsets: [Vec<u32>; 6],
}

// TODO: try if using bitflags would speed this up
pub struct Parser2 {
    pub track_measures: bool,
    pub track_sections: bool,
    pub track_offsets: bool,
}
impl Default for Parser2 {
    fn default() -> Self {
        Self {
            track_measures: true,
            track_sections: true,
            track_offsets: true,
        }
    }
}
pub trait ParserInput<'a>: std::iter::Iterator<Item = &'a str> {}
impl<'a, T: std::iter::Iterator<Item = &'a str>> ParserInput<'a> for T {}

impl Parser2 {
    // TODO: add a way to discard measure/part information for backends that don't need it
    // Will probably involve a restructuring of the parsing step to be controlled by the backend.
    // I imagine a Parser {settings: ParserSettings }.parse()
    pub fn parse<'a>(&self, lines: impl ParserInput<'a>) -> Result<Parse2Result, BackendError<'a>> {
        let mut diagnostics = vec![];
        let mut sections = Vec::with_capacity(10);
        let mut part_buf = Vec::with_capacity(6);
        let mut part_start_tick = 0;
        let mut strings: [Vec<RawTick>; 6] = [const { Vec::new() }; 6];
        let mut string_measures: [Vec<Measure>; 6] = [const { Vec::new() }; 6];
        let mut offsets: [Vec<u32>; 6] = [const { Vec::new() }; 6];
        let mut string_names = ['\0'; 6];
        let mut source_offset = 0u32;
        for (line_idx, line) in lines.enumerate() {
            if line.trim().is_empty() {
                if !part_buf.is_empty() {
                    diagnostics.push(Diagnostic::warn(
                        ErrorLocation::LineOnly(line_idx),
                        DiagnosticKind::EmptyLineInPart,
                    ));
                }
                source_offset += line.len() as u32 + 1;
                continue;
            }

            if let Ok((rem, comment)) = comment_line(line) {
                // I don't think there is a way to write an invalid comment after a valid start, just to be safe
                assert!(rem.is_empty(), "Invalid comment syntax (line {line_idx})");
                if !part_buf.is_empty() {
                    diagnostics.push(Diagnostic::warn(
                        ErrorLocation::LineOnly(line_idx),
                        DiagnosticKind::CommentInPart,
                    ));
                }
                sections.push(Section::Comment(comment.to_string()));
            } else {
                match partline(
                    line,
                    line_idx,
                    source_offset,
                    &mut strings[part_buf.len()],
                    &mut string_measures[part_buf.len()],
                    &mut offsets[part_buf.len()],
                ) {
                    Ok((rem, line)) => {
                        if !rem.is_empty() {
                            return Err(BackendError {
                                // the measure with the problem is the first that is not parsed
                                main_location: ErrorLocation::LineAndMeasure(line_idx, line.len()),
                                relevant_lines: line_idx..=line_idx,
                                kind: BackendErrorKind::InvalidPartlineSyntax(rem),
                            });
                        }

                        string_names[part_buf.len()] = line.string_name;
                        part_buf.push(line);
                        if part_buf.len() == 6 {
                            // This part is for correcting multichar frets (fret >=10)
                            // because the parser will errorneously generate two rests
                            // when there's a multichar fret on another string
                            if let Err((kind, invalid_offset, invalid_line_idx)) = fixup_part(
                                part_start_tick,
                                &mut part_buf,
                                &mut strings,
                                &mut string_measures,
                                &offsets,
                                &string_names,
                            ) {
                                return Err(BackendError {
                                    main_location: ErrorLocation::SourceOffset(SourceOffset::new(
                                        invalid_offset,
                                    )),
                                    relevant_lines: invalid_line_idx..=invalid_line_idx,
                                    kind,
                                });
                            }
                            // flush part buf
                            sections.push(Section::Part {
                                part: part_buf.try_into().unwrap(),
                            });
                            part_buf = Vec::with_capacity(6);
                            part_start_tick = strings[0].len();
                        }
                    }
                    Err(x) => {
                        return Err(BackendError {
                            main_location: ErrorLocation::LineOnly(line_idx),
                            relevant_lines: line_idx..=line_idx,
                            kind: BackendErrorKind::InvalidPartlineSyntax(x),
                        });
                    }
                }
            }

            // +1 for \n
            source_offset += line.len() as u32 + 1;
        }
        Ok(Parse2Result {
            diagnostics,
            sections,
            measures: string_measures,
            strings,
            string_names,
            offsets,
        })
    }
}

fn fixup_part(
    // we only need to check after this
    start_tick: usize,
    part: &mut [Partline],
    strings: &mut [Vec<RawTick>; 6],
    measures: &mut [Vec<Measure>; 6],
    offsets: &[Vec<u32>; 6],
    string_names: &[char; 6],
) -> Result<(), (BackendErrorKind<'static>, usize, usize)> {
    let (mut tick_count, track_with_least_ticks) = strings
        .iter()
        .enumerate()
        .map(|(track_idx, track)| (track.len(), track_idx))
        .min() // the string with the least ticks has the most twochar frets
        .expect("Empty score");
    let mut tick_idx = start_tick;
    while tick_idx < tick_count {
        let Some((
            multichar_t_idx,
            RawTick {
                element: TabElement::Fret(multichar_fret),
                ..
            },
        )) = ({
            strings
                .iter()
                .enumerate()
                .map(|(t_idx, track)| {
                    (
                        t_idx,
                        track.get(tick_idx).unwrap_or_else(|| {
                            panic!(
                                "String {} doesn't have tick {tick_idx}\n",
                                string_names[t_idx]
                            );
                        }),
                    )
                })
                .find(|(_, x)| match x.element {
                    TabElement::Fret(x) => x >= 10,
                    _ => false,
                })
        })
        else {
            tick_idx += 1;
            continue;
        };
        // so we stop borrowing strings
        let multichar_fret = *multichar_fret;
        // This is a multi-char tick. Remove adjacent rest everywhere where it is not
        // multi-char.
        for string_idx in 0..6 {
            let tick_onechar_on_this_track = match strings[string_idx][tick_idx].element {
                TabElement::Fret(x) => x < 10,
                TabElement::Rest | TabElement::DeadNote => true,
            };
            if tick_onechar_on_this_track {
                let idx_to_remove = [tick_idx + 1, tick_idx - 1].into_iter().find(|x| {
                    strings[string_idx]
                        .get(*x)
                        //.inspect(|x| println!("affine: {x:?}"))
                        .is_some_and(|y| y.element == TabElement::Rest)
                });
                let Some(idx_to_remove) = idx_to_remove else {
                    return Err((
                        BackendErrorKind::BadMulticharTick {
                            multichar: (string_names[multichar_t_idx], multichar_fret),
                            invalid: (
                                string_names[string_idx],
                                strings[string_idx][tick_idx].element.clone(),
                            ),
                            tick_idx: tick_idx as u32,
                        },
                        offsets[string_idx][tick_idx + 1] as usize,
                        string_idx,
                    ));
                };
                strings[string_idx].remove(idx_to_remove);

                // now also update measure information to stay correct
                for measure_idx in part[string_idx].measures.clone() {
                    let mc = &mut measures[string_idx][measure_idx].content;
                    if *mc.start() > tick_idx {
                        // move measure to the right
                        *mc = mc.start() - 1..=mc.end() - 1;
                    } else if *mc.end() > tick_idx {
                        // pop one from end
                        *mc = *mc.start()..=mc.end() - 1
                    }
                }
                if string_idx == track_with_least_ticks {
                    tick_count -= 1;
                }
            }
        }
        tick_idx += 1;
    }
    Ok(())
}
#[test]
fn test_parse2() {
    let parser = Parser2::default();
    let i1 = r#"
e|---|
B|-3-|
G|6-6|
D|---|
A|---|
E|---|

// This is a comment

e|---|
B|3-3|
G|-6-|
D|---|
A|---|
E|---|"#;
    insta::assert_debug_snapshot!(parser.parse(i1.lines()));
    let i2 = r#"
e|---|
B|-3-|
G|6-6|
D|---|
A|-o-|
E|---|

// This is a comment

e|---|
B|3-3|
G|-6-|
D|---|
A|---|
E|---|"#;
    insta::assert_debug_snapshot!(parser.parse(i2.lines()));

    let i3 = r#"
e|-------------12---------------------|
B|-------------3---0--------------3---|
G|---------0-2-------2-0--------------|
D|---0-2-3---------------3-2-0--------|
A|-3---------------------------3------|
E|------------------------------------|"#;
    insta::assert_debug_snapshot!(parser.parse(i3.lines()));
    let i3 = r#"
e|-------------12---------------------|
B|-------------3---0--------------3---|
G|---------0-2-------2-0--------------|
D|---0-2-3---------------3-2-0--------|
A|-3---------------------------3------|
E|0-----------------------------------|"#;
    insta::assert_debug_snapshot!(parser.parse(i3.lines()));
}
