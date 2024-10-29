use super::MuxmlNote2;
use crate::backend::errors::backend_error::BackendError;

pub fn get_fretboard_note2(string: char, fret: u8) -> Result<MuxmlNote2, BackendError<'static>> {
    return Ok(MuxmlNote2 {
        step: STRING_BASE_NOTES[string as usize]
            .unwrap_or_else(|| panic!("Don't know base note for string {string}"))
            + fret as u8,
        dead: false,
    });
}

/// TODO fill in everything with u8s for no extra cost and more flexibility
/// Generated by:
/// let mut lookup: [Option<u8>; 256] = [None; 256];
/// lookup['D' as usize] = Some(3 * 12 + 2);
/// lookup['E' as usize] = Some(3 * 12 + 4);
/// lookup['A' as usize] = Some(3 * 12 + 9);
/// lookup['D' as usize] = Some(4 * 12 + 2);
/// lookup['G' as usize] = Some(4 * 12 + 7);
/// lookup['B' as usize] = Some(4 * 12 + 11);
/// lookup['e' as usize] = Some(5 * 12 + 4);
/// println!("{lookup:?}")
#[rustfmt::skip]
const STRING_BASE_NOTES: [Option<u8>;256]= [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(45), Some(59), None, Some(50), Some(40), None, Some(55), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(64), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];
