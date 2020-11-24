use core::convert::TryFrom;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Key {
    One = b'1',
    Two = b'2',
    Three = b'3',
    Four = b'4',
    Five = b'5',
    Six = b'6',
    Seven = b'7',
    Eight = b'8',
    Nine = b'9',
    Zero = b'0',
}

#[derive(Copy, Clone, Debug)]
pub struct XT(pub u8);

impl TryFrom<XT> for Key {
    type Error = &'static str;
    fn try_from(value: XT) -> Result<Self, Self::Error> {
        match value.0 {
            0x02 => Ok(Key::One),
            0x03 => Ok(Key::Two),
            0x04 => Ok(Key::Three),
            0x05 => Ok(Key::Four),
            0x06 => Ok(Key::Five),
            0x07 => Ok(Key::Six),
            0x08 => Ok(Key::Seven),
            0x09 => Ok(Key::Eight),
            0x0a => Ok(Key::Nine),
            0x0b => Ok(Key::Zero),
            byte => Err("Unknown character"),
        }
    }
}
