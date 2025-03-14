use super::{ConversionError, DataType};
use serde::{Deserialize, Serialize};

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

    fn bytes_to_string(&self, data: &[u8]) -> Result<String, ConversionError> {
        if data.len() != self.get_size() {
            return Err(ConversionError::SizeError);
        }

        let cstr = std::ffi::CStr::from_bytes_until_nul(data)
            .map_err(|_e| ConversionError::CStrUntilNullError)?;
        Ok(cstr.to_string_lossy().into_owned())
    }
}
impl StrDataType {
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
    pub fn with_size(mut self, size: usize) -> Self {
        self.set_size(size);
        self
    }
}
