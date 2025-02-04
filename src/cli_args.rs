use std::fmt::Display;

use clap::{Parser, Subcommand};
use guitar_tab::backend::fixup::{FixupBackendSettings, FixupDumpOptions};
use guitar_tab::backend::{
    //format::{FormatBackendSettings, FormatDumpOptions},
    muxml,
    BackendSelector,
};

#[derive(Parser)]
#[command(
    version,
    name = "guitar_tab",
    about = "Transforms a melody in guitar tab notation into a score in standard music notation"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Don't write out diagnostics that were produced. Makes execution slightly faster
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// The most complex backend, usually produces the best results but is slower than the others
    /// and in more cases cannot work over imperfections of a bad tab
    #[command(visible_alias = "musicxml", long_about = "")]
    Muxml {
        /// A lot of tabs will leave rest before/after the measure content for better clarity.
        /// This option will remove those.
        #[arg(short = 'm', long)]
        trim_measure: bool,

        /// A lot of tabs will leave a rest between each note, even when it is not needed to
        /// discriminate between single- and double-digit frets. This will remove these,
        /// effectively transforming the IR [1,rest,2,rest,3,rest,4,rest,5] of `e|1-2-3-4-5|`
        /// into [1,2,3,4,5]
        #[arg(short = 'n', long)]
        remove_rest_between_notes: bool,
        #[arg(short = 't', long)]
        /// Simplify time signature, e.g. 8/8 -> 4/4
        simplify_time_signature: bool,
        input_path: String,
        output_path: String,
    },
    /// The simplest backend, with no fancy features. Use this for playback, as importing its output into a music score application will result in an ugly score
    Midi { input_path: String, output_path: String },

    /// Tries to fix errors in the score, until it can be parsed.
    Fixup {
        input_path: String,
        output_path: String,
        /// Dump the parse tree
        #[arg(value_enum, short = 'd', long)]
        dump: Option<FixupDumpOptions>,
    },
}

impl Commands {
    pub fn input_path(&self) -> &str {
        match self {
            Commands::Muxml { input_path, .. }
            //| Commands::Muxml { input_path, .. }
            | Commands::Midi { input_path, .. } => input_path,
            | Commands::Fixup { input_path, .. } => input_path,
        }
    }

    pub fn output_path(&self) -> &str {
        match self {
            Commands::Muxml { output_path, .. }
            //| Commands::Muxml { output_path, .. }
            | Commands::Midi { output_path, .. } => output_path,
              | Commands::Fixup { output_path, .. } => output_path,
        }
    }

    pub fn to_backend_selector(&self) -> BackendSelector {
        match self {
            Commands::Muxml {
                trim_measure,
                remove_rest_between_notes,
                simplify_time_signature,
                ..
            } => BackendSelector::Muxml2(muxml::settings::Settings {
                remove_rest_between_notes: *remove_rest_between_notes,
                trim_measure: *trim_measure,
                simplify_time_signature: *simplify_time_signature,
            }),
            Commands::Midi { .. } => BackendSelector::Midi(()),
            Commands::Fixup { dump, .. } => {
                BackendSelector::Fixup(FixupBackendSettings { dump: dump.clone() })
            }
        }
    }
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Commands::Muxml { .. } => "muxml2",
                //Commands::Muxml { .. } => "muxml",
                Commands::Fixup { .. } => "fixup",
                Commands::Midi { .. } => "midi",
            }
        )
    }
}
