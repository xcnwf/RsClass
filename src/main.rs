fn main() {
    println!("Hello, world!");
    let a = 1;
}

mod types {

    pub trait DataType {
        fn get_size(&self) -> usize;
        fn get_name(&self) -> String;
        fn from_bytes(&self, data: &[u8]) -> Result<String, ()>;
    }

    struct BooleanDataType {
        size: usize,
    }

    impl DataType for BooleanDataType {
        fn get_size(&self) -> usize {
            self.size
        }
        fn get_name(&self) -> String {
            "Boolean".to_string()
        }
        fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
            if data.len() != self.size {
                return Err(());
            }

            Ok((u8[0] == 1).to_string())
        }
    }

    #[derive(Debug)]
    pub enum Endian {
        Little,
        Big,
    }

    struct IntegerDataType {
        t: Type,
        name: String,
        endian: Endian,
        signed: bool,
        hex: bool,
    }

    impl DataType for IntegerDataType {
        fn get_size(&self) -> usize {
            self.size
        }
        fn get_name(&self) -> String {
            self.name
        }
        fn from_bytes(&self, data: &[u8]) -> Result<String, ()> {
            if data.len() != self.size {
                return Err(());
            }
        }
    }
}
