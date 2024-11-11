use super::errors::{backend_error::BackendError, diagnostic::Diagnostic};
use crate::{
    backend::Backend,
    parser::{parser2::Parse2Result, Section},
    rlen,
};

pub struct FormatBackend();

#[derive(Clone)]
pub struct FormatBackendSettings {
    pub dump: bool,
}

impl Backend for FormatBackend {
    type BackendSettings = FormatBackendSettings;

    fn process<Out: std::io::Write>(
        parse_result: Parse2Result,
        out: &mut Out,
        settings: Self::BackendSettings,
    ) -> Result<Vec<Diagnostic>, BackendError> {
        if settings.dump {
            println!("{parse_result:?}")
        }
        let diagnostics = vec![];
        let mut formatted = String::new();
        let mut measure_cnt = 0;
        for section in parse_result.sections {
            match section {
                Section::Part { part, .. } => {
                    let measures_in_part = rlen(&part[0].measures);
                    for measure_idx in 0..measures_in_part {
                        formatted += &format!("// SYS: Measure {}\n", measure_cnt + 1);
                        for (l_idx, line) in part.iter().enumerate() {
                            formatted.push(line.string_name);
                            formatted.push('|');
                            formatted += &parse_result.measures[l_idx][measure_idx]
                                .print_pretty_string(&parse_result.strings[l_idx]);
                            formatted.push('|');
                            formatted.push('\n');
                        }
                        formatted.push('\n');
                        measure_cnt += 1;
                    }
                }
                Section::Comment(x) => {
                    // SYS-comments were generated during a previous format run, so don't include
                    // them
                    if !x.trim_start().starts_with("SYS:") {
                        formatted += "//";
                        formatted += &x;
                        formatted.push('\n');
                    }
                }
            }
        }

        if let Err(x) = out.write_all(formatted.as_bytes()) {
            return Err(BackendError::from_io_error(x, diagnostics));
        }

        Ok(diagnostics)
    }
}
