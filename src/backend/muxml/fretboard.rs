use crate::backend::errors::backend_error::BackendError;

const NOTE2_STEPS: [(char, bool); 12] = [
    ('C', false),
    ('C', true),
    ('D', false),
    ('D', true),
    ('E', false),
    ('F', false),
    ('F', true),
    ('G', false),
    ('G', true),
    ('A', false),
    ('A', true),
    ('B', false),
];
#[derive(Debug)]
pub struct MuxmlNote2 {
    /// Numeric representation of the frequency.
    ///
    /// step=0 is an octave 0 C,
    /// step=1 is an octave 0 C#,
    /// step=2 is an octave 0 D,
    /// and so on.
    ///
    /// Can represent 20 full octaves which should be plenty.
    pub step: u8,
    pub dead: bool,
}
impl MuxmlNote2 {
    pub fn step_octave_sharp(&self) -> (char, u8, bool) {
        let stepidx = (self.step % 12) as usize;
        let octave = self.step / 12;
        (NOTE2_STEPS[stepidx].0, octave, NOTE2_STEPS[stepidx].1)
    }
}

// TODO: make this panic an error
pub fn get_fretboard_note2(string: char, fret: u8) -> Result<MuxmlNote2, BackendError> {
    Ok(MuxmlNote2 {
        step: STRING_BASE_NOTES[string as usize]
            .unwrap_or_else(|| panic!("Don't know base note for string {string}"))
            + fret,
        dead: false,
    })
}

/// TODO: benchmark this against a match and a hashmap, a hashmap will be probably faster as it can
/// stay in cache (we need a total of max 16 base notes*1 byte => 16bytes which is less than a
/// cache line
/// Generated by tools/gen_note_lookup.rs
#[rustfmt::skip]
const STRING_BASE_NOTES: [Option<u8>;256]= [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(45), Some(59), None, Some(50), Some(40), None, Some(55), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(64), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];
