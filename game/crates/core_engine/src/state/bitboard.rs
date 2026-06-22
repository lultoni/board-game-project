//! 64-bit bitboard newtype. One bit per square on the 8×8 board.
//!
//! Square indexing: bit `i` corresponds to square `i` in row-major order,
//! `i = rank * 8 + file`, rank 0 = bottom row (player 1 side), file 0 = left.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Bitboard(0);
    pub const FULL: Self  = Bitboard(!0);

    #[inline]
    pub fn from_square(sq: u8) -> Self {
        debug_assert!(sq < 64);
        Bitboard(1u64 << sq)
    }

    #[inline]
    pub fn contains(self, sq: u8) -> bool {
        (self.0 >> sq) & 1 == 1
    }

    #[inline]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn lsb(self) -> Option<u8> {
        if self.0 == 0 { None } else { Some(self.0.trailing_zeros() as u8) }
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;
    #[inline] fn bitand(self, rhs: Self) -> Self { Bitboard(self.0 & rhs.0) }
}
impl std::ops::BitOr for Bitboard {
    type Output = Self;
    #[inline] fn bitor(self, rhs: Self) -> Self { Bitboard(self.0 | rhs.0) }
}
impl std::ops::BitXor for Bitboard {
    type Output = Self;
    #[inline] fn bitxor(self, rhs: Self) -> Self { Bitboard(self.0 ^ rhs.0) }
}
impl std::ops::Not for Bitboard {
    type Output = Self;
    #[inline] fn not(self) -> Self { Bitboard(!self.0) }
}
