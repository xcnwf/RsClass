use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serde::{Serialize, Deserialize};
use enum_dispatch::enum_dispatch;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Copy, Clone)]
enum ArchSize {
    #[default]
    Arch32,
    Arch64
}
impl ArchSize {
    pub fn get_size(&self) -> usize {
        match self {
            ArchSize::Arch32 => 4,
            ArchSize::Arch64 => 8
        }
    }
}

thread_local! {
    static ARCH_SIZE: std::cell::Cell<ArchSize> = std::cell::Cell::new(ArchSize::Arch32);
}

// ENDIANNESS
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum Endianness {
    Big,
    #[default]
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

#[enum_dispatch]
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

#[enum_dispatch(DataType)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataTypeEnum {
    IntegerDataType,
    BooleanDataType,
    FloatDataType,
    StrDataType,
    StructDataType,
    PointerDataType,
    ArrayDataType,
    //FUNCTIONS 
    //CLASS (with VTABLES)
}


/* BOOLEANS */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BooleanDataType {
    size: usize,
}
impl Default for BooleanDataType {
    fn default() -> Self {
        BooleanDataType {size: 1 }
    }
}
impl DataType for BooleanDataType {
    fn get_size(&self) -> usize {
        self.size
    }
    fn get_name(&self) -> String {
        return "Boolean".into();
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

/* INTEGERS */
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum IntSize {
    Integer8,
    Integer16,
    #[default]
    Integer32,
    Integer64,
}
impl Into<usize> for IntSize {
    fn into(self) -> usize {
        use IntSize::*;
        match self {
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
        use IntSize::*;
        match value {
            1 => Ok(Integer8),
            2 => Ok(Integer16),
            4 => Ok(Integer32),
            8 => Ok(Integer64),
            _ => Err("That size is not valid, can only be powers of 2 between 1 and 8.")
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
        IntegerDataType{size: IntSize::Integer32, signed: false, hex: false, endianness: Endianness::Little}
    }
}
impl DataType for IntegerDataType {
    fn get_size(&self) -> usize {
        return self.size.into();
    }
    fn get_name(&self) -> String {
        "Integer".into()
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

/* FLOATS */
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub enum FloatPrecision {
    #[default]
    Simple,
    Double,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FloatDataType {
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
        "Float".into()
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


/* STRINGS */
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct StrDataType {
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

/* STRUCTS */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructDataType {
    name: String,
    entries: Vec<StructEntry>,
}
impl Default for StructDataType {
    fn default() -> Self {
        Self {name: "STRUCT".into(), entries: Vec::new()}
    }
}
impl DataType for StructDataType {
    fn get_size(&self) -> usize {
        self.entries.iter().map(|e| e.size).sum()
    }
    fn get_name(&self) -> String {
        return self.name.clone();
    }
    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.get_size() {
            return Err(());
        }

        let mut reprs = Vec::new();
        for e in &self.entries {
            reprs.push(format!("{}: {}",e.name, e.datatype.from_bytes(&data[e.offset..e.offset+e.size])?));
        }

        let s = format!("{} {{ {} }}", self.get_name(), reprs.join(", "));

        Ok(s)
    }
}
impl StructDataType {
    pub fn new(name: String , entries : Vec<StructEntry>) -> Self {
        StructDataType {name , entries}   
    }

    pub fn get_entries(&self) -> &Vec<StructEntry> {
        &self.entries
    }
    pub fn get_mut_entries(&mut self) -> &mut Vec<StructEntry> {
        &mut self.entries
    }
    pub fn push_entry(&mut self, mut e: StructEntry) {
        e.offset = self.entries.last().map(|e| e.offset + e.datatype.get_size()).unwrap_or(0 );
        self.entries.push(e);
    }
    pub fn insert_entry(&mut self, idx: usize, mut e: StructEntry) {   
        todo!("Insert entry")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructEntry {
    name: String,
    size: usize,
    offset: usize,
    datatype: DataTypeEnum,
}
impl StructEntry {
    pub fn new(name: String, datatype: DataTypeEnum) -> Self {
        Self{name, size: datatype.get_size(), offset: 0usize, datatype}
    }
    pub fn set_name(&mut self , name: String) { self.name = name; }
    pub fn get_name(&self) -> &String { &self.name }
    pub fn get_datatype(&self) -> &DataTypeEnum { &self.datatype }
    pub fn set_dataype(&mut self , datatype: DataTypeEnum) { 
        self.size = datatype.get_size();
        self.datatype = datatype;
    }
    pub fn get_size (&self) -> usize { self.size }   
}

/* POINTER */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerDataType {
    pointed_datatype: Box<DataTypeEnum>
}
impl DataType for PointerDataType {
    fn get_size(&self) -> usize {
        ARCH_SIZE.get().get_size()
    }

    fn get_name(&self) -> String {
        format!("Pointer to {}", self.pointed_datatype.get_name())
    }

    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.get_size() {
            return Err(());
        }

        let cstr = std::ffi::CStr::from_bytes_until_nul(data).map_err(|_| ())?;
        cstr.to_owned().into_string().map_err(|_| ())
    }
}

/* ARRAY */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrayDataType {
    element_datatype: Box<DataTypeEnum>,
    size: usize,
}
impl DataType for ArrayDataType {
    fn get_size(&self) -> usize {
        self.element_datatype.get_size() * self.size
    }

    fn get_name(&self) -> String {
        format!("Array of {}", self.element_datatype.get_name())
    }

    fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
        if data.len() != self.get_size() {
            return Err(());
        }

        let cstr = std::ffi::CStr::from_bytes_until_nul(data).map_err(|_| ())?;
        cstr.to_owned().into_string().map_err(|_| ())
    }
}



/* TESTS */
#[cfg(test)]
mod test {
    use crate::typing::DataType;

    use super::*;

    #[test]
    fn test_boolean_zero() {
        let dt = BooleanDataType {
            size: 4,
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
            size: IntSize::Integer8,
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
            size: IntSize::Integer32,
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
            size: IntSize::Integer32,
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
            precision: FloatPrecision::Simple,
            endianness: Endianness::Big,
        };

        let data = [0x3f, 0x80, 0x00, 0x00];
        assert_eq!(dt.get_size(), 4);
        assert_eq!(dt.from_bytes(&data)?, "1.000");
        Ok(())
    }
}
