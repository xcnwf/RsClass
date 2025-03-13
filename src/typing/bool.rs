use super::{ConversionError, DataType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BooleanDataType {
    size: usize,
}
impl Default for BooleanDataType {
    fn default() -> Self {
        BooleanDataType { size: 1 }
    }
}
impl DataType for BooleanDataType {
    fn get_size(&self) -> usize {
        self.size
    }
    fn get_name(&self) -> String {
        "Boolean".into()
    }
    fn bytes_to_string(&self, data: &[u8]) -> Result<String, ConversionError> {
        if data.len() != self.size {
            return Err(ConversionError::SizeError);
        }

        let mut b = false;
        for &x in &data[0..self.get_size()] {
            b |= x != 0u8;
        }
        Ok((b).to_string())
    }
}
impl BooleanDataType {
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
    pub fn with_size(mut self, size: usize) -> Self {
        self.set_size(size);
        self
    }
}
