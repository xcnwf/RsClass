use super::{ConversionError, DataType, Endianness};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum IntSize {
    Integer8,
    Integer16,
    #[default]
    Integer32,
    Integer64,
}
impl From<IntSize> for usize {
    fn from(val: IntSize) -> Self {
        use IntSize::{Integer16, Integer32, Integer64, Integer8};
        match val {
            Integer8 => 1,
            Integer16 => 2,
            Integer32 => 4,
            Integer64 => 8,
        }
    }
}
impl TryFrom<usize> for IntSize {
    type Error = &'static str;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use IntSize::{Integer16, Integer32, Integer64, Integer8};
        match value {
            1 => Ok(Integer8),
            2 => Ok(Integer16),
            4 => Ok(Integer32),
            8 => Ok(Integer64),
            _ => Err("That size is not valid, can only be powers of 2 between 1 and 8."),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegerDataType {
    size: IntSize,
    signed: bool,
    hex: bool,
    endianness: Endianness,
}
impl Default for IntegerDataType {
    fn default() -> Self {
        IntegerDataType {
            size: IntSize::Integer32,
            signed: false,
            hex: false,
            endianness: Endianness::Little,
        }
    }
}
impl DataType for IntegerDataType {
    fn get_size(&self) -> usize {
        self.size.into()
    }
    fn get_name(&self) -> String {
        "Integer".into()
    }

    fn bytes_to_string(&self, data: &[u8]) -> Result<String, ConversionError> {
        if data.len() != self.get_size() {
            return Err(ConversionError::SizeError);
        };
        let val = match self.size {
            IntSize::Integer8 => u64::from(data[0]),
            _ => match self.endianness {
                Endianness::Little => LittleEndian::read_uint(data, self.get_size()),
                Endianness::Big => BigEndian::read_uint(data, self.get_size()),
            },
        };

        let s = match (self.hex, self.signed) {
            (true, _) => format!("{val:#X}"),
            (false, true) => {
                let p = 8 * self.get_size();
                let mut signed_val = val;
                if (val >> (p - 1)) == 1 {
                    signed_val |= u64::MAX ^ ((1 << p) - 1);
                }
                format!("{}", signed_val as i64)
            }
            (false, false) => format!("{val}"),
        };
        Ok(s)
    }
}
impl IntegerDataType {
    pub fn set_hex(&mut self, hex: bool) {
        self.hex = hex;
    }
    pub fn toggle_hex(&mut self) {
        self.set_hex(!self.hex);
    }
    pub fn with_hex(mut self, hex: bool) -> Self {
        self.set_hex(hex);
        self
    }

    pub fn set_signed(&mut self, signed: bool) {
        self.signed = signed;
    }
    pub fn toggle_signed(&mut self) {
        self.set_signed(!self.signed);
    }
    pub fn with_signed(mut self, signed: bool) -> Self {
        self.set_signed(signed);
        self
    }

    pub fn set_endianness(&mut self, endianness: Endianness) {
        self.endianness = endianness;
    }
    pub fn toggle_endianness(&mut self) {
        self.set_endianness(self.endianness.toggle());
    }
    pub fn with_endianness(mut self, endianness: Endianness) -> Self {
        self.set_endianness(endianness);
        self
    }

    pub fn set_size(&mut self, size: IntSize) {
        self.size = size;
    }
    pub fn with_size(mut self, size: IntSize) -> Self {
        self.set_size(size);
        self
    }
}
