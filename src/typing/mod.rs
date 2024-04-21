use byteorder::{BigEndian, ByteOrder, LittleEndian};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Endianness {
    Big,
    Little,
}

impl Endianness {
    fn toggle(self) -> Endianness {
        use Endianness::{Big, Little};
        match self {
            Little => Big,
            Big => Little,
        }
    }
}

pub trait DataType {
    fn get_size(&self) -> usize;
    fn get_name(&self) -> String;
    fn from_bytes(&self, data: &[u8]) -> Result<String, ()>;

    fn clone_box(&self) -> Box<dyn DataType>
    where
        Self: 'static + Clone,
    {
        Box::new(self.clone())
    }
}

pub enum DataTypeEnum {
    Simple(Box<dyn SimpleDataType>),
    Composite(Box<dyn CompositeDataType>),
    Pointer(Box<DataTypeEnum>),
}

struct Entry {
    name: String,
    datatype: DataTypeEnum,
}

pub trait SimpleDataType: DataType {
    fn has_hex(&self) -> bool {
        false
    }
    fn get_hex(&self) -> Option<bool> {
        None
    }
    fn set_hex(&mut self, is_hex: bool) {}
    fn toggle_hex(&mut self) {
        self.set_hex(self.get_hex().unwrap_or(false));
    }
    fn has_endianness(&self) -> bool {
        false
    }
    fn get_endianness(&self) -> Option<Endianness> {
        None
    }
    fn set_endianness(&mut self, ed: Endianness) {}
    fn toggle_endianness(&mut self) {
        self.set_endianness(
            self.get_endianness()
                .map_or(Endianness::Little, Endianness::toggle),
        );
    }
}

pub trait CompositeDataType: DataType {
    fn get_children(&self) -> Vec<DataTypeEnum>;
}

#[derive(Clone)]
struct BooleanDataType {
    size: usize,
    name: String,
}

#[derive(Clone)]
struct IntegerDataType {
    size: usize,
    signed: bool,
    hex: bool,
    endianness: Endianness,
    name: String,
}

impl DataType for BooleanDataType {
    fn get_size(&self) -> usize {
        self.size
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.size {
            return Err(());
        }

        let mut b = false;
        for &x in &data[0..self.get_size()] {
            b |= x != 0u8;
        }
        Ok((b).to_string())
    }
}
impl SimpleDataType for BooleanDataType {}

impl DataType for IntegerDataType {
    fn get_size(&self) -> usize {
        return self.size;
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.get_size() {
            return Err(());
        };
        let val = match self.get_size() {
            1 => Ok(data[0] as u64),
            2..=8 => match self.endianness {
                Endianness::Little => Ok(LittleEndian::read_uint(data, self.get_size())),
                Endianness::Big => Ok(BigEndian::read_uint(data, self.get_size())),
            },
            _ => Err(()),
        }?;

        let s = match (self.hex, self.signed) {
            (true, true) => format!("{:x}", val as i64),
            (true, false) => format!("{:x}", val),
            (false, true) => format!("{}", val as i64),
            (false, false) => format!("{}", val),
        };
        Ok(s)
    }
}
impl SimpleDataType for IntegerDataType {
    fn has_endianness(&self) -> bool {
        true
    }
    fn has_hex(&self) -> bool {
        true
    }
    fn set_hex(&mut self, is_hex: bool) {
        self.hex = is_hex;
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum FloatPrecision {
    Simple,
    Double,
}

#[derive(Clone)]
struct FloatDataType {
    name: String,
    endianness: Endianness,
    precision: FloatPrecision,
}

impl DataType for FloatDataType {
    fn get_size(&self) -> usize {
        use FloatPrecision::*;
        match self.precision {
            Simple => 4,
            Double => 8,
        }
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.get_size() {
            return Err(());
        }

        use Endianness::*;
        match self.precision {
            FloatPrecision::Simple => {
                let val = match self.endianness {
                    Big => BigEndian::read_f32(data),
                    Little => LittleEndian::read_f32(data),
                };
                Ok(val.to_string())
            }
            FloatPrecision::Double => {
                let val = match self.endianness {
                    Big => BigEndian::read_f64(data),
                    Little => LittleEndian::read_f64(data),
                };
                Ok(val.to_string())
            }
        }
    }
}

impl SimpleDataType for StrDataType {}

#[derive(Clone)]
struct StrDataType {
    size: usize,
}

impl DataType for StrDataType {
    fn get_size(&self) -> usize {
        self.size
    }

    fn get_name(&self) -> String {
        String::from("Null terminated string")
    }

    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.get_size() {
            return Err(());
        }

        let cstr = std::ffi::CStr::from_bytes_until_nul(data).map_err(|_| ())?;
        cstr.to_owned().into_string().map_err(|_| ())
    }
}

#[cfg(test)]
mod test {
    use crate::typing::DataType;

    use super::BooleanDataType;

    #[test]
    fn test_boolean_zero() {
        let dt = BooleanDataType {
            size: 4,
            name: String::new(),
        };
        let data = [0; 4];

        assert_eq!(dt.get_size(), 4);
        let val = dt.from_bytes(&data).unwrap();
        assert_eq!(val, "false");
    }

    #[test]
    fn test_boolean_not_zero() {
        let dt = BooleanDataType {
            size: 4,
            name: String::new(),
        };
        let mut data = [0; 4];
        data[2] = 5;

        assert_eq!(dt.get_size(), 4);
        let val = dt.from_bytes(&data).unwrap();
        assert_eq!(val, "true");
    }
}

// macro_rules! integer_data_type {
//     ($name:ident, $type:ident) => {
//         paste! {
//             pub struct [<$name DataType>] {}
//             impl DataType for [<$name DataType>] {
//                 fn get_size(&self) -> usize {
//                     core::mem::size_of::<$type>()
//                 }
//                 fn get_name(&self) -> String {
//                     stringify!($name).to_string()
//                 }
//                 fn from_bytes(&self, data: &[u8]) -> Result <String, ()> {
//                     if data.len() != self.get_size() {return Err(())};
//                     let val = LittleEndian::read_int(data, self.get_size()) as $type;
//
//                     Ok(val.to_string())
//                 }
//             }
//         }
//     };
// }
//
// integer_data_type![Char, i8];
// integer_data_type!(Byte, u8);
// integer_data_type!(Int16, i16);
// integer_data_type!(WORD, u16);
// integer_data_type!(DWORD, u32);
// integer_data_type!(Int32, i32);
// integer_data_type!(Int64, i64);
// integer_data_type!(QWORD, u64);
//
// integer_data_type!(Float, f32);
// integer_data_type!(Double, f64);
//#[derive(Debug)]
//pub enum Endian {
//    Little,
//    Big,
//}

struct DataTypeSettings {}
