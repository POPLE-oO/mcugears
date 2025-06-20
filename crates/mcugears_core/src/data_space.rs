// ルートから読み込み
use crate::*;

// DataSpace操作
pub trait DataSpace {
    fn new() -> Self;
    fn write_to(&mut self, address: DataAddress, value: RegisterSize);
    fn read_from(&self, address: DataAddress) -> RegisterSize;
}

// data space操作対象
pub enum DataAddress {
    Byte { address: RegisterSize },
}

#[cfg(test)]
pub mod test_utilities {
    use super::*;

    pub struct ExampleDataSpace(Vec<u8>);

    impl ExampleDataSpace {
        const DATA_SPACE_SIZE: usize = 0x8FF;
    }

    impl DataSpace for ExampleDataSpace {
        fn new() -> Self {
            Self(vec![0; Self::DATA_SPACE_SIZE + 1])
        }

        fn write_to(&mut self, address: DataAddress, value: RegisterSize) {
            match address {
                DataAddress::Byte { address } => self.0[address as usize] = value as u8,
            };
        }

        fn read_from(&self, address: DataAddress) -> RegisterSize {
            match address {
                DataAddress::Byte { address } => self.0[address as usize] as RegisterSize,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utilities::*;
    use super::*;

    // ---  RAMの読み書き  ---
    #[cfg(test)]
    mod test_data_space_write_read {
        use super::*;

        // ---  write_to,read_fromの実行
        #[test]
        fn test_write_read() {
            let mut data_space = ExampleDataSpace::new();
            data_space.write_to(DataAddress::Byte { address: 510 }, 134);
            assert_eq!(
                data_space.read_from(DataAddress::Byte { address: 510 }),
                134
            );
        }
    }
}
