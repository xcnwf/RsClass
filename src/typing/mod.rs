use byteorder::{BigEndian, ByteOrder, LittleEndian};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

static BUILTINS_DT: Vec<DataTypeEnum> = vec![
    DataTypeEnum::Simple(Box::new(BooleanDataType {
        name: String::from("bool8"),
        size: 1,
    })),
    DataTypeEnum::Simple(Box::new(BooleanDataType {
        name: String::from("bool16"),
        size: 2,
    })),
    DataTypeEnum::Simple(Box::new(BooleanDataType {
        name: String::from("bool32"),
        size: 4,
    })),
    DataTypeEnum::Simple(Box::new(IntegerDataType {
        name: String::from("char"),
        size: 1,
        endianness: Endianness::Little,
        hex: false,
        signed: true,
    })),
    DataTypeEnum::Simple(Box::new(BooleanDataType {
        name: String::from("word"),
        size: 2,
    })),
];

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
    Simple(dyn SimpleDataType),
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

#[derive(Clone, Debug)]
struct BooleanDataType {
    size: usize,
    name: String,
}

#[derive(Clone, Debug)]
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
            (true, _) => format!("{:#X}", val),
            (false, true) => {
                let p = 8 * self.get_size();
                let mut signed_val = val;
                if (val >> (p - 1)) == 1 {
                    signed_val = (u64::MAX ^ ((1 << p) - 1)) | signed_val;
                }
                format!("{}", signed_val as i64)
            }
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

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
enum FloatPrecision {
    Simple,
    Double,
}

#[derive(Clone, Debug)]
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
                Ok(format!("{:.3}", val))
            }
            FloatPrecision::Double => {
                let val = match self.endianness {
                    Big => BigEndian::read_f64(data),
                    Little => LittleEndian::read_f64(data),
                };
                Ok(format!("{:.3}", val))
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

    use super::{BooleanDataType, Endianness, FloatDataType, FloatPrecision, IntegerDataType};

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

    #[test]
    fn u8() -> Result<(), ()> {
        let dt = IntegerDataType {
            name: String::new(),
            size: 1,
            signed: false,
            hex: false,
            endianness: Endianness::Big,
        };

        let data = [50; 1];

        assert_eq!(dt.get_size(), 1);
        assert_eq!(dt.endianness, Endianness::Big);
        assert_eq!(dt.from_bytes(&data)?, "50");
        Ok(())
    }

    #[test]
    fn h32() -> Result<(), ()> {
        let dt = IntegerDataType {
            name: String::new(),
            size: 4,
            signed: true,
            hex: true,
            endianness: Endianness::Little,
        };

        let data = [0xEF, 0xBE, 0xAD, 0xDE];

        assert_eq!(dt.get_size(), 4);
        assert_eq!(dt.endianness, Endianness::Little);
        assert_eq!(dt.from_bytes(&data)?, "0xDEADBEEF");
        Ok(())
    }

    #[test]
    fn i32_minus_one() -> Result<(), ()> {
        let dt = IntegerDataType {
            name: String::new(),
            size: 4,
            signed: true,
            hex: false,
            endianness: Endianness::Little,
        };

        let data = [0xFF, 0xFF, 0xFF, 0xFF];

        assert_eq!(dt.get_size(), 4);
        assert_eq!(dt.endianness, Endianness::Little);
        assert_eq!(dt.from_bytes(&data)?, "-1");
        Ok(())
    }

    #[test]
    fn double() -> Result<(), ()> {
        let dt = FloatDataType {
            name: String::new(),
            precision: FloatPrecision::Simple,
            endianness: Endianness::Big,
        };

        let data = [0x3f, 0x80, 0x00, 0x00];
        assert_eq!(dt.get_size(), 4);
        assert_eq!(dt.from_bytes(&data)?, "1.000");
        Ok(())
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
