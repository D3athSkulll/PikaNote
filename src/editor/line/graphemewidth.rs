
#[derive(Clone, Copy)]
pub enum GraphemeWidth {
    Half,
    Full,
}

type GraphemeIdx = usize;
type ByteIdx=usize; // new typedefs for better understanding of function returntypes
impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}