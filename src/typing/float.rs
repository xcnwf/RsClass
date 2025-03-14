use super::{ConversionError, DataType, Endianness};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub enum FloatPrecision {
    #[default]
    Simple,
    Double,
}
impl FloatPrecision {
    pub fn toggle(&self) -> Self {
        use FloatPrecision::{Double, Simple};
        match self {
            Simple => Double,
            Double => Simple,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FloatDataType {
    endianness: Endianness,
    precision: FloatPrecision,
}
impl DataType for FloatDataType {
    fn get_size(&self) -> usize {
        use FloatPrecision::{Double, Simple};
        match self.precision {
            Simple => 4,
            Double => 8,
        }
    }
    fn get_name(&self) -> String {
        "Float".into()
    }
    fn bytes_to_string(&self, data: &[u8]) -> Result<String, ConversionError> {
        if data.len() != self.get_size() {
            return Err(ConversionError::SizeError);
        }

        use Endianness::{Big, Little};
        match self.precision {
            FloatPrecision::Simple => {
                let val = match self.endianness {
                    Big => BigEndian::read_f32(data),
                    Little => LittleEndian::read_f32(data),
                };
                Ok(format!("{val:.3}"))
            }
            FloatPrecision::Double => {
                let val = match self.endianness {
                    Big => BigEndian::read_f64(data),
                    Little => LittleEndian::read_f64(data),
                };
                Ok(format!("{val:.3}"))
            }
        }
    }
}
impl FloatDataType {
    pub fn set_precision(&mut self, precision: FloatPrecision) {
        self.precision = precision;
    }
    pub fn toggle_precision(&mut self) {
        self.set_precision(self.precision.toggle());
    }
    pub fn with_precision(mut self, precision: FloatPrecision) -> Self {
        self.set_precision(precision);
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
}
