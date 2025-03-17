use std::ops::Deref;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageFlags(u16);

impl MessageFlags {
    pub const NONE: MessageFlags = MessageFlags(0);
    pub const COMPRESSED: MessageFlags = MessageFlags(1 << 0);
    pub const ENCRYPTED: MessageFlags = MessageFlags(1 << 1);
    pub const REQUIRES_ACK: MessageFlags = MessageFlags(1 << 2);
    pub const HAS_PAYLOAD: MessageFlags = MessageFlags(1 << 3);

    #[inline]
    pub fn contains(self, other: MessageFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for MessageFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        MessageFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for MessageFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        MessageFlags(self.0 & rhs.0)
    }
}

impl AsRef<u16> for MessageFlags {
    fn as_ref(&self) -> &u16 {
        &self.0
    }
}

impl Deref for MessageFlags {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<u16> for MessageFlags {
    fn from(flags: u16) -> Self {
        MessageFlags(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags() {
        let flag = MessageFlags::COMPRESSED | MessageFlags::ENCRYPTED;

        assert!(flag.contains(MessageFlags::ENCRYPTED));
        assert!(flag.contains(MessageFlags::COMPRESSED));

        assert!(!flag.contains(MessageFlags::HAS_PAYLOAD));
    }
}
