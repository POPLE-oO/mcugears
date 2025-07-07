// userのアクセスできるram
pub trait UserRam {
    // UserRamのスタートアドレス
    const START_ADDRESS: usize;
    // UserRamの終了アドレス
    const END_ADDRESS: usize;

    // 初期化
    fn new() -> Self;

    // 書き込み
    fn write_to(&mut self, address: RamAddress, value: usize) -> &mut Self;
    //読み込み
    fn read_from(&mut self, address: RamAddress) -> usize;

    // startアドレス取得
    fn get_start_address() -> usize {
        Self::START_ADDRESS
    }
    // endアドレス取得
    fn get_end_address() -> usize {
        Self::END_ADDRESS
    }
}
// Ramのアドレス
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RamAddress(pub usize);

//  テスト
#[cfg(test)]
pub mod user_ram_tests {
    use super::*;

    // utility
    // RAM構造体
    #[derive(Clone, PartialEq, Debug)]
    pub struct ExampleUserRam(Vec<u8>);

    impl UserRam for ExampleUserRam {
        // UserRamのスタートアドレス
        const START_ADDRESS: usize = 0x0100;
        // UserRamの終了アドレス
        const END_ADDRESS: usize = 0x08FF;

        // 初期化関数
        fn new() -> Self {
            ExampleUserRam(vec![0; Self::END_ADDRESS + 1])
        }

        fn write_to(&mut self, address: RamAddress, value: usize) -> &mut Self {
            self.0[address.0] = value as u8;
            self
        }

        fn read_from(&mut self, address: RamAddress) -> usize {
            self.0[address.0] as usize
        }
    }

    // user_ram初期化
    #[cfg(test)]
    mod initialize {
        use super::*;

        #[test]
        fn initialize() {
            // 初期化
            let user_ram = ExampleUserRam::new();

            // テスト
            assert_eq!(user_ram, ExampleUserRam(vec![0; 0x8FF + 1]))
        }
    }

    // user_ramの操作
    mod operation {
        use super::*;
        use rstest::rstest;

        // 読み込み
        #[test]
        fn read() {
            // 初期化
            let mut user_ram = ExampleUserRam::new();

            // 書き込み
            user_ram.0[0x1FF] = 21;

            // テスト
            assert_eq!(user_ram.read_from(RamAddress(0x1FF)), 21);
        }

        // 書き込み
        #[rstest]
        #[case::default(0x1F3, 110, 110)]
        #[case::truncate(0x300, 420, 164)]
        fn write(#[case] address: usize, #[case] value: usize, #[case] expected: usize) {
            // 初期化
            let mut user_ram = ExampleUserRam::new();

            // 書き込み
            user_ram.write_to(RamAddress(address), value);

            // テスト
            assert_eq!(user_ram.read_from(RamAddress(address)), expected);
        }
    }
}
