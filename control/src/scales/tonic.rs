// ALLOW: Any of the invariants may be constructed using `try_from`.
#[allow(dead_code)]
#[repr(usize)]
#[derive(Clone, Copy, Debug, defmt::Format, PartialEq)]
pub enum Tonic {
    C = 0,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Tonic {
    pub const LAST_TONIC: Self = Self::B;

    pub fn index(self) -> u8 {
        self as u8
    }
}

impl TryFrom<usize> for Tonic {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index <= Self::LAST_TONIC.index() as usize {
            Ok(unsafe { core::mem::transmute::<usize, Self>(index) })
        } else {
            Err(())
        }
    }
}
