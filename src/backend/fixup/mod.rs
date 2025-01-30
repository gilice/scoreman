use std::borrow::Cow;
use std::cmp::PartialEq;
use std::io::Write;
use std::time::{Duration, Instant};

use clap::ValueEnum;

use super::BackendResult;
use crate::backend::errors::backend_error::BackendError;
use crate::backend::errors::backend_error_kind::BackendErrorKind;
use crate::backend::errors::diagnostic::Diagnostic;
use crate::backend::errors::diagnostic_kind::DiagnosticKind;
use crate::backend::errors::error_location::ErrorLocation;
use crate::parser::char;
use crate::parser::parser3::parse3;
use crate::{backend::Backend, rlen, time, traceln};

pub struct FixupBackend();
#[derive(ValueEnum, Clone)]
pub enum FixupDumpOptions {
    AST,
    PrettyTracks,
}
#[derive(Clone)]
pub struct FixupBackendSettings {
    pub dump: Option<FixupDumpOptions>,
}

struct LocationTracker {
    pub data: [ErrorLocation; 5],
    pub push_cnt: u32,
}

impl LocationTracker {
    pub fn new() -> Self {
        Self { data: [const { ErrorLocation::NoLocation }; 5], push_cnt: 0 }
    }
    pub fn add(&mut self, l: ErrorLocation) {
        traceln!("location_tracker:: before add: {:?}", self.data);
        // shift left
        for i in 0..4 {
            self.data[i] = self.data[i + 1].clone();
        }
        self.data[4] = l;
        traceln!("location_tracker:: after add: {:?}", self.data);
        self.push_cnt += 1;
    }
    pub fn is_same(&self) -> bool {
        self.push_cnt > 4 && self.data.windows(2).all(|a| a[0] == a[1])
    }
}
impl Backend for FixupBackend {
    type BackendSettings = FixupBackendSettings;

    fn process<Out: std::io::Write>(
        parser_input: &[String], out: &mut Out, settings: Self::BackendSettings,
    ) -> BackendResult {
        let mut diagnostics = vec![];
        // TODO: figure out a way not to clone these
        let mut parser_input = parser_input.to_owned();
        let mut parse_time = Duration::from_secs(0);
        let mut fixup_start = Instant::now();
        let mut location_tracker = LocationTracker::new();
        loop {
            let parse_start = Instant::now();
            let parsed = parse3(&parser_input);
            parse_time = parse_start.elapsed();
            match &parsed.error {
                None => break,
                Some(err) => {
                    location_tracker.add(err.main_location.clone());
                    if location_tracker.is_same() {
                        let lines =
                            err.main_location.get_line_idx().map(|x| x..=x).unwrap_or(0..=0);
                        return BackendResult::new(
                            diagnostics,
                            Some(BackendError::fixup_failed(err.main_location.clone(), lines)),
                            Some(parse_time),
                            Some(fixup_start.elapsed()),
                        );
                    }
                    match err.kind {
                        BackendErrorKind::IOError(_) | BackendErrorKind::FmtError(_) => {
                            return BackendResult::new(
                                diagnostics,
                                parsed.error,
                                Some(parse_time),
                                None,
                            );
                        }
                        BackendErrorKind::InvalidStringName => {}
                        BackendErrorKind::EmptyScore => {}
                        BackendErrorKind::NoClosingBarline => {
                            let l_idx = err.main_location.get_line_idx().unwrap();
                            let line = &mut parser_input[l_idx];
                            line.truncate(line.trim_end().len());
                            line.push('|');
                            diagnostics.push(Diagnostic::info(
                                err.main_location.clone(),
                                DiagnosticKind::FormatAddedBarline,
                            ))
                        }
                        BackendErrorKind::FixupFailed => unreachable!(),
                        BackendErrorKind::Parse3InvalidCharacter(x) => {
                            let Some((line_idx, char_idx)) = err
                                .main_location
                                .get_line_idx()
                                .zip(err.main_location.get_char_idx())
                            else {
                                // TODO: try looking up the last parsed character here
                                panic!()
                            };
                            parser_input[line_idx].replace_range(char_idx..char_idx + 1, "-");
                            diagnostics.push(Diagnostic::info(
                                err.main_location.clone(),
                                DiagnosticKind::FormatReplacedInvalid,
                            ))
                        }
                    };
                }
            }
        }
        let gen_time = fixup_start.elapsed();
        let maybe_io_err =
            out.write_all(parser_input.join("\n").as_ref()).map_err(BackendError::from).err();
        BackendResult::new(diagnostics, maybe_io_err, Some(parse_time), Some(gen_time))
    }
}
