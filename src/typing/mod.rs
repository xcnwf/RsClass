pub mod bool;
pub use bool::BooleanDataType;
pub mod int;
pub use int::{IntSize, IntegerDataType};
pub mod float;
pub use float::{FloatDataType, FloatPrecision};
pub mod str;
pub use str::StrDataType;
pub mod struct_dt;
pub use struct_dt::{StructDataType, StructEntry};

use std::fmt::Display;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Copy, Clone)]
enum ArchSize {
    #[default]
    Arch32,
    Arch64,
}
impl ArchSize {
    pub fn get_size(&self) -> usize {
        match self {
            ArchSize::Arch32 => 4,
            ArchSize::Arch64 => 8,
        }
    }
}

thread_local! {
    static ARCH_SIZE: std::cell::Cell<ArchSize> = const { std::cell::Cell::new(ArchSize::Arch32) };
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

#[derive(Debug, Clone, Copy)]
pub enum ConversionError {
    SizeError,
    CStrUntilNullError,
    NotConvertibleError,
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ConversionError::SizeError => {
                "Provided size does not match with the size of the datatype."
            }
            ConversionError::CStrUntilNullError => {
                "The data does not contain a null terminating byte."
            }
            ConversionError::NotConvertibleError => "This datatype cannot be converted to string.",
        };
        write!(f, "{}", txt)
    }
}

impl std::error::Error for ConversionError {}

#[enum_dispatch]
pub trait DataType {
    fn get_size(&self) -> usize;
    fn get_name(&self) -> String;
    fn bytes_to_string(&self, data: &[u8]) -> Result<String, ConversionError>;

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
    //Enums TODO!
    //FUNCTIONS
    //CLASS (with VTABLES)
}

/* BOOLEANS */

/* POINTER */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerDataType {
    pointed_datatype: Box<DataTypeEnum>,
}
impl DataType for PointerDataType {
    fn get_size(&self) -> usize {
        ARCH_SIZE.get().get_size()
    }

    fn get_name(&self) -> String {
        format!("Pointer to {}", self.pointed_datatype.get_name())
    }

    fn bytes_to_string(&self, _data: &[u8]) -> Result<String, ConversionError> {
        Err(ConversionError::NotConvertibleError)
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

    fn bytes_to_string(&self, data: &[u8]) -> Result<String, ConversionError> {
        if data.len() != self.get_size() {
            return Err(ConversionError::SizeError);
        }

        todo!();
    }
}

/* TESTS */
#[cfg(test)]
mod test {
    use crate::typing::DataType;

    use super::*;

    #[test]
    fn test_boolean_zero() {
        let dt = BooleanDataType::default().with_size(4);
        let data = [0; 4];

        assert_eq!(dt.get_size(), 4);
        let val = dt.bytes_to_string(&data).unwrap();
        assert_eq!(val, "false");
    }

    #[test]
    fn test_boolean_not_zero() {
        let dt = BooleanDataType::default().with_size(4);
        let mut data = [0; 4];
        data[2] = 5;

        assert_eq!(dt.get_size(), 4);
        let val = dt.bytes_to_string(&data).unwrap();
        assert_eq!(val, "true");
    }

    #[test]
    fn u8() -> Result<(), ()> {
        let dt = IntegerDataType::default()
            .with_size(IntSize::Integer8)
            .with_signed(false)
            .with_hex(false)
            .with_endianness(Endianness::Big);

        let data = [50; 1];

        assert_eq!(dt.get_size(), 1);
        assert_eq!(dt.bytes_to_string(&data).expect("Should succeed"), "50");
        Ok(())
    }

    #[test]
    fn h32() -> Result<(), ()> {
        let dt = IntegerDataType::default()
            .with_size(IntSize::Integer32)
            .with_signed(true)
            .with_hex(true)
            .with_endianness(Endianness::Little);

        let data = [0xEF, 0xBE, 0xAD, 0xDE];

        assert_eq!(dt.get_size(), 4);
        assert_eq!(
            dt.bytes_to_string(&data).expect("Should succeed"),
            "0xDEADBEEF"
        );
        Ok(())
    }

    #[test]
    fn i32_minus_one() -> Result<(), ()> {
        let dt = IntegerDataType::default()
            .with_size(IntSize::Integer32)
            .with_signed(true)
            .with_hex(false)
            .with_endianness(Endianness::Little);

        let data = [0xFF, 0xFF, 0xFF, 0xFF];

        assert_eq!(dt.get_size(), 4);
        assert_eq!(dt.bytes_to_string(&data).expect("Should succeed"), "-1");
        Ok(())
    }

    #[test]
    fn double() -> Result<(), ()> {
        let dt = FloatDataType::default()
            .with_precision(FloatPrecision::Simple)
            .with_endianness(Endianness::Big);

        let data = [0x3f, 0x80, 0x00, 0x00];
        assert_eq!(dt.get_size(), 4);
        assert_eq!(dt.bytes_to_string(&data).expect("Should succeed"), "1.000");
        Ok(())
    }
}
