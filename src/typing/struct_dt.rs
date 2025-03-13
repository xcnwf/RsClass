use super::{ConversionError, DataType, DataTypeEnum};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructDataType {
    name: String,
    entries: Vec<StructEntry>,
}
impl Default for StructDataType {
    fn default() -> Self {
        Self {
            name: "STRUCT".into(),
            entries: Vec::new(),
        }
    }
}
impl DataType for StructDataType {
    fn get_size(&self) -> usize {
        self.entries.iter().map(|e| e.size).sum()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn bytes_to_string(&self, _data: &[u8]) -> Result<String, ConversionError> {
        Err(ConversionError::NotConvertibleError)
    }
}
impl StructDataType {
    pub fn new(name: String, entries: Vec<StructEntry>) -> Self {
        StructDataType { name, entries }
    }
    pub fn get_entries(&self) -> &Vec<StructEntry> {
        &self.entries
    }
    pub fn push_entry(&mut self, mut e: StructEntry) {
        e.offset = self
            .entries
            .last()
            .map_or(0, |e| e.offset + e.datatype.get_size());
        self.entries.push(e);
    }
    pub fn insert_entry(&mut self, _idx: usize, _e: StructEntry) {
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
        Self {
            name,
            size: datatype.get_size(),
            offset: 0usize,
            datatype,
        }
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_datatype(&self) -> &DataTypeEnum {
        &self.datatype
    }
    pub fn set_dataype(&mut self, datatype: DataTypeEnum) {
        self.size = datatype.get_size();
        self.datatype = datatype;
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
}
