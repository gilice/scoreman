pub mod parser2;
mod parser3;
#[cfg(test)]
mod parser_tests;

use std::ops::RangeInclusive;

use nom::{bytes::complete::is_not, error::VerboseError, sequence::preceded, IResult, Parser};

use nom_supreme::tag::complete::tag;

use crate::rlen;

#[derive(Debug, PartialEq)]
pub struct Score(pub Vec<Section>);

type VerboseResult<Input, Parsed> = IResult<Input, Parsed, VerboseError<Input>>;

#[derive(Debug, PartialEq)]
pub enum Section {
    Part { part: [Partline; 6] },
    Comment(String),
}

fn comment_line(s: &str) -> VerboseResult<&str, &str> {
    preceded(tag("//"), is_not("\n\r")).parse(s)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Partline {
    pub string_name: char,
    /// which measures originate from this partline in the measure buf of string_name
    pub measures: RangeInclusive<usize>,
}
impl Partline {
    /// Returns the measure count of this partline
    pub fn len(&self) -> usize {
        rlen(&self.measures)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
/// like `e|--------------4-----------|-----0--------------5-----|`
/// If called with append_to, the returned Partline will have no measures itself
fn partline<'a>(
    s: &'a str,
    parent_line_idx: usize,
    string_buf: &mut Vec<RawTick>,
    string_measure_buf: &mut Vec<Measure>,
) -> Result<(&'a str, Partline, usize), &'a str> {
    let (rem, string_name) = string_name()(s)?;
    let (mut rem, _) = char('|')(rem)?;
    let mut last_parsed_idx = 1;
    let mut measures = string_measure_buf.len()..=string_measure_buf.len();
    let mut tick_cnt = 0;

    while !rem.is_empty() {
        let mut measure = Measure {
            content: string_buf.len()..=string_buf.len(),
            parent_line: parent_line_idx,
            index_on_parent_line: rlen(&measures),
        };
        loop {
            let rl_before = rem.len();
            let Ok(x) = tab_element(rem) else { break };
            rem = x.0;
            last_parsed_idx += rl_before - rem.len(); // multichar frets
            string_buf.push(RawTick {
                element: x.1,
                parent_line: parent_line_idx,
                idx_on_parent_line: last_parsed_idx,
            });
            measure.extend_1();
            tick_cnt += 1;
        }
        measure.content = *measure.content.start()..=measure.content.end() - 1;
        string_measure_buf.push(measure);
        measures = *measures.start()..=measures.end() + 1;
        rem = char('|')(rem)?.0;
        last_parsed_idx += 1;
    }
    // off by one: because we are using inclusive ranges, for example the first line, with only 1
    // measure, would be 0..=1 but we want it to be 0..=0
    measures = *measures.start()..=measures.end() - 1;
    Ok((
        rem,
        Partline {
            string_name,
            measures,
        },
        tick_cnt,
    ))
}

/// A staff of a single string.
/// like `|--------------4-----------|`
/// The string it is on is encoded out-of-band
#[derive(Debug, PartialEq, Clone)]
pub struct Measure {
    /// The indices of the track this measure is on which belong to this measure
    pub content: RangeInclusive<usize>,
    pub parent_line: usize,
    pub index_on_parent_line: usize,
}

impl Measure {
    pub fn extend_1(&mut self) {
        self.content = *self.content.start()..=self.content.end() + 1
    }
    pub fn pop_1(&mut self) {
        self.content = *self.content.start()..=self.content.end() - 1
    }
    pub fn len(&self) -> usize {
        rlen(&self.content)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get_content<'a>(&self, string_buf: &'a [RawTick]) -> &'a [RawTick] {
        &string_buf[self.content.clone()]
    }
    pub fn print_pretty_string(&self, string_buf: &[RawTick]) -> String {
        let mut pretty = String::new();
        for x in self.content.clone() {
            match string_buf[x].element {
                TabElement::Fret(x) => pretty += &x.to_string(),
                TabElement::Rest => pretty += "-",
                TabElement::DeadNote => pretty += "x",
            }
        }
        pretty
    }
}
fn char(c: char) -> impl Fn(&str) -> Result<(&str, char), &str> {
    move |s: &str| match s.chars().next() {
        Some(cc) if cc == c => Ok((&s[1..], c)),
        _ => Err(s),
    }
}

fn string_name() -> impl Fn(&str) -> Result<(&str, char), &str> {
    move |s: &str| match s.chars().next() {
        Some(c) if c.is_alphabetic() => Ok((&s[1..], c)),
        _ => Err(s),
    }
}
fn tab_element(s: &str) -> Result<(&str, TabElement), &str> {
    let mut chars = s.chars();
    match chars.next() {
        Some('-') => Ok((&s[1..], TabElement::Rest)),
        Some('x') => Ok((&s[1..], TabElement::DeadNote)),
        Some(c) if c.is_numeric() => {
            let mut len = 1;
            // 123a
            for cc in chars {
                if !cc.is_numeric() {
                    break;
                }
                len += 1;
            }
            match s[0..len].parse() {
                Ok(x) => Ok((&s[len..], TabElement::Fret(x))),
                Err(_) => Err(s),
            }
        }
        Some(_) | None => Err(s),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RawTick {
    pub element: TabElement,
    pub idx_on_parent_line: usize,
    pub parent_line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TabElement {
    Fret(u8),
    Rest,
    DeadNote,
}
