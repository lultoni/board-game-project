//! Mailbox entry: packed u16 carrying per-square piece data.
//!
//! Bit layout (LSB → MSB):
//!   bits 0..2   HP            (2 bits, 0..=2)
//!   bits 2..4   Armor         (2 bits, 0..=2; Stack M cap is 2, the 4th value is unused)
//!   bits 4..7   Combo counter (3 bits, 0..=7)
//!   bits 7..11  Skill 1 ID    (4 bits, 0..=15; 0 = unequipped sentinel)
//!   bits 11..15 Skill 2 ID    (4 bits, 0..=15)
//!   bit  15     reserved

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MailboxEntry(pub u16);

pub const EMPTY_MAILBOX_ENTRY: MailboxEntry = MailboxEntry(0);

impl MailboxEntry {
    #[inline] pub fn hp(self)           -> u8 { ((self.0)        & 0b11)        as u8 }
    #[inline] pub fn armor(self)        -> u8 { ((self.0 >> 2)   & 0b11)        as u8 }
    #[inline] pub fn combo(self)        -> u8 { ((self.0 >> 4)   & 0b111)       as u8 }
    #[inline] pub fn skill1(self)       -> u8 { ((self.0 >> 7)   & 0b1111)      as u8 }
    #[inline] pub fn skill2(self)       -> u8 { ((self.0 >> 11)  & 0b1111)      as u8 }

    #[inline]
    pub fn with_hp(self, hp: u8) -> Self {
        debug_assert!(hp <= 2);
        MailboxEntry((self.0 & !0b11) | (hp as u16 & 0b11))
    }
    #[inline]
    pub fn with_armor(self, a: u8) -> Self {
        debug_assert!(a <= 2, "Stack M armor cap is 2");
        MailboxEntry((self.0 & !(0b11 << 2)) | ((a as u16 & 0b11) << 2))
    }
    #[inline]
    pub fn with_combo(self, c: u8) -> Self {
        debug_assert!(c <= 7);
        MailboxEntry((self.0 & !(0b111 << 4)) | ((c as u16 & 0b111) << 4))
    }
    #[inline]
    pub fn with_skill1(self, s: u8) -> Self {
        debug_assert!(s <= 15);
        MailboxEntry((self.0 & !(0b1111 << 7)) | ((s as u16 & 0b1111) << 7))
    }
    #[inline]
    pub fn with_skill2(self, s: u8) -> Self {
        debug_assert!(s <= 15);
        MailboxEntry((self.0 & !(0b1111 << 11)) | ((s as u16 & 0b1111) << 11))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_and_unpack_roundtrip() {
        let e = MailboxEntry::default()
            .with_hp(2)
            .with_armor(2)
            .with_combo(5)
            .with_skill1(7)
            .with_skill2(15);
        assert_eq!(e.hp(),     2);
        assert_eq!(e.armor(),  2);
        assert_eq!(e.combo(),  5);
        assert_eq!(e.skill1(), 7);
        assert_eq!(e.skill2(), 15);
    }
}
