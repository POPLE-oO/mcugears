// ビット操作
pub trait BitOperation {
    fn get_bit(&self, id: usize) -> bool;
    fn generate_from_bit(&self, flags: &[Option<bool>]) -> usize;
}
impl BitOperation for usize {
    // 右からid番目(0スタート)のbit取得
    fn get_bit(&self, id: usize) -> bool {
        (*self & (1 << id)) != 0
    }

    // boolのOptionからSome(1)なら1,Some(0)なら0,Noneなら元の値のままでbit操作
    fn generate_from_bit(&self, flags: &[Option<bool>]) -> usize {
        let mut result = *self;
        // flagsの若いidのほうからbitの左側
        for (id, flag) in flags.iter().rev().enumerate() {
            // 変更するなら
            if let Some(bit) = flag {
                if *bit {
                    // trueなら1
                    result |= 1 << id;
                } else {
                    // falseなら0
                    result &= !(1 << id);
                }
            }
            // 変更しない場合は操作しない
        }

        result
    }
}

// マクロ
// 演算書き込み実装のマクロ
macro_rules! impl_operation {
    ($fn_name:ident, $op:ident) => {
        fn $fn_name(&mut self, register_type: RegisterType, value: usize) -> &mut Self {
            // 演算
            self.write_to(register_type, self.read_from(register_type).$op(value))
        }
    };
}

// レジスタを表す構造体
pub trait Registers {
    // 初期化
    fn new() -> Self;
    // 書き込み
    fn write_to(&mut self, register_type: RegisterType, value: usize) -> &mut Self;
    // 読み込み
    fn read_from(&self, register_type: RegisterType) -> usize;

    // 加算
    impl_operation!(add_to, wrapping_add);
    // 減算
    impl_operation!(sub_from, wrapping_sub);
    // 乗算
    impl_operation!(mul_to, wrapping_mul);
    // 徐算
    impl_operation!(div_from, wrapping_div);
}

// レジスタ種類を表す列挙型
#[derive(Clone, Copy)]
pub enum RegisterType {
    General { id: usize },
    Status,
    StackPointer,
    ProgramCounter,
    Io { id: usize },
}

// プログラムカウンター更新
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PCUpdate {
    Default,
    Relative(isize),
    Absolute(usize),
}

// 実行後レジスタ更新
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegisterUpdate {
    cycles: usize,
    pc_update: PCUpdate,
}

impl RegisterUpdate {
    // 初期化
    pub fn new(cycles: usize, pc_update: PCUpdate) -> Self {
        RegisterUpdate { cycles, pc_update }
    }

    // register updateを用いたレジスタ更新
    // fn update<R: Registers>(&self, registers: &mut R) {
    //     registers.update_timer(self.cycles);
    //     registers.update_pc(self.pc_update);
    // }
}

#[cfg(test)]
pub mod register_tests {
    use super::*;
    use rstest::rstest;

    // utility
    // レジスタ構造体
    #[derive(Clone, Debug, PartialEq)]
    pub struct ExampleRegisters {
        general: [u8; 32],
        status: u8,
        stack_pointer: u16,
        program_counter: u16,
        io: [u8; 256],
    }

    // レジスタの実装
    impl Registers for ExampleRegisters {
        // 初期化
        fn new() -> Self {
            // 0初期化
            ExampleRegisters {
                general: [0; 32],
                status: 0,
                stack_pointer: 0,
                program_counter: 0,
                io: [0; 256],
            }
        }

        // レジスタ書き込み
        fn write_to(&mut self, register_type: RegisterType, value: usize) -> &mut Self {
            // 書き込み
            match register_type {
                RegisterType::General { id } => self.general[id] = value as u8,
                RegisterType::Status => self.status = value as u8,
                RegisterType::StackPointer => self.stack_pointer = value as u16,
                RegisterType::ProgramCounter => self.program_counter = value as u16,
                RegisterType::Io { id } => self.io[id] = value as u8,
            }

            self
        }

        // レジスタ読み取り
        fn read_from(&self, register_type: RegisterType) -> usize {
            // 読み取った値を返す
            match register_type {
                RegisterType::General { id } => self.general[id].into(),
                RegisterType::Status => self.status.into(),
                RegisterType::StackPointer => self.stack_pointer.into(),
                RegisterType::ProgramCounter => self.program_counter.into(),
                RegisterType::Io { id } => self.io[id].into(),
            }
        }
    }

    // registersの初期化
    #[cfg(test)]
    mod initialize {
        use super::*;

        #[test]
        fn initialize() {
            let registers = ExampleRegisters::new();

            assert_eq!(
                registers,
                ExampleRegisters {
                    general: [0; 32],
                    status: 0,
                    stack_pointer: 0,
                    program_counter: 0,
                    io: [0; 256],
                }
            )
        }
    }

    // 読み書き操作テスト
    #[cfg(test)]
    mod operation {
        use super::*;
        use rstest::rstest;

        // 書き込み
        #[test]
        fn write() {
            // 初期化
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::General { id: 14 };

            // 書き込み操作実行
            registers.write_to(register_type, 140);

            // 想定している結果
            let mut expected = ExampleRegisters {
                general: [0; 32],
                status: 0,
                stack_pointer: 0,
                program_counter: 0,
                io: [0; 256],
            };
            expected.general[14] = 140;

            // テスト
            assert_eq!(registers, expected);
        }

        // 読み取り
        #[test]
        fn read() {
            // 初期化
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::General { id: 11 };
            registers.write_to(register_type, 24);

            // 読み込み操作実行
            let value = registers.read_from(register_type);

            // テスト
            assert_eq!(value, 24);
        }

        // 様々なレジスタの種類の読み書きに対応
        #[rstest]
        #[case::general(RegisterType::General{id:2}, 200)]
        #[case::status(RegisterType::Status, 121)]
        #[case::stack_pointer(RegisterType::StackPointer, 528)]
        #[case::program_counter(RegisterType::ProgramCounter, 1204)]
        #[case::io(RegisterType::Io{id:105}, 21)]
        fn write_read_variously(#[case] register_type: RegisterType, #[case] value: usize) {
            // 初期化
            let mut registers = ExampleRegisters::new();

            //書き込み,読み込み
            let result = registers
                .write_to(register_type, value)
                .read_from(register_type);

            // テスト
            assert_eq!(result, value);
        }

        // 境界上の読み書きテスト
        #[rstest]
        #[case::general_min(RegisterType::General{id:0}, 12)]
        #[case::general_max(RegisterType::General{id:31}, 42)]
        #[case::io_min(RegisterType::Io{id:0}, 110)]
        #[case::io_max(RegisterType::Io{id:255}, 223)]
        fn read_write_on_boundary(#[case] register_type: RegisterType, #[case] value: usize) {
            // 初期化
            let mut registers = ExampleRegisters::new();

            //書き込み,読み込み
            let result = registers
                .write_to(register_type, value)
                .read_from(register_type);

            // テスト
            assert_eq!(result, value);
        }

        // 境界外の書きテスト
        #[rstest]
        #[case::general_max(RegisterType::General{id:32}, 117)]
        #[case::io_max(RegisterType::Io{id:256}, 98)]
        #[should_panic]
        fn write_out_of_boundary(#[case] register_type: RegisterType, #[case] value: usize) {
            // 初期化
            let mut registers = ExampleRegisters::new();

            //書き込み
            registers.write_to(register_type, value);
        }

        // 境界外の読みテスト
        #[rstest]
        #[case::general_max(RegisterType::General{id:32})]
        #[case::io_max(RegisterType::Io{id:256})]
        #[should_panic]
        fn read_out_of_boundary(#[case] register_type: RegisterType) {
            // 初期化
            let registers = ExampleRegisters::new();

            //読み込み
            registers.read_from(register_type);
        }

        // 切り捨て処理
        #[rstest]
        #[case::general(RegisterType::General{id:22}, 310, 54)]
        #[case::status(RegisterType::Status, 288, 32)]
        #[case::stack_pointer(RegisterType::StackPointer, 65635, 99)]
        #[case::program_counter(RegisterType::ProgramCounter, 66222, 686)]
        #[case::io(RegisterType::Io{id:28}, 400, 144)]
        fn write_read_truncation(
            #[case] register_type: RegisterType,
            #[case] value: usize,
            #[case] expected: usize,
        ) {
            // 初期化
            let mut registers = ExampleRegisters::new();

            //書き込み,読み込み
            let result = registers
                .write_to(register_type, value)
                .read_from(register_type);

            // テスト
            assert_eq!(result, expected);
        }
    }

    // 四則演算操作のテスト
    #[cfg(test)]
    mod calculation {
        use super::*;

        // 演算テスト用マクロ
        macro_rules! impl_operation_test {
            ($test_name:ident, $op:ident$(,#[case::$pattern:ident($reg_type:expr,$val:expr,$expected:expr)])+) => {
                #[rstest]
                $(
                    #[case::$pattern($reg_type,$val,$expected)]
                )+
                fn $test_name(
                    #[case] register_type: RegisterType,
                    #[case] value: usize,
                    #[case] expected: usize,
                ) {
                    // 初期化
                    let mut registers = ExampleRegisters::new();
                    registers.write_to(register_type, 100);

                    // 操作
                    let result = registers
                        .$op(register_type, value)
                        .read_from(register_type);

                    // テスト
                    assert_eq!(result, expected);
                }
            };
        }

        // 加算テスト
        impl_operation_test!(add, add_to,
            #[case::default(RegisterType::General{id:30}, 63, 163)],
            #[case::truncate(RegisterType::General{id:11}, 250, 94)]
        );

        // 減算テスト
        impl_operation_test!(sub, sub_from,
            #[case::default(RegisterType::General{id:13}, 12, 88)],
            #[case::truncate(RegisterType::General{id:7}, 108, 248)]
        );

        // 乗算テスト
        impl_operation_test!(mul, mul_to,
            #[case::default(RegisterType::General{id:4}, 2, 200)],
            #[case::truncate(RegisterType::General{id:24}, 7, 188)]
        );

        // 徐算テスト
        impl_operation_test!(div, div_from,
            #[case::default(RegisterType::General{id:8}, 4, 25)],
            #[case::truncate(RegisterType::General{id:20}, 1000, 0)]
        );
    }

    // ビット操作のテスト
    #[cfg(test)]
    mod bit_operation {
        use super::*;

        // ビット取得
        #[rstest]
        #[case::get_1(0b0000_1000, 3, true)]
        #[case::get_0(0b0000_0010, 5, false)]
        fn get_bit(#[case] value: usize, #[case] id: usize, #[case] expected: bool) {
            assert_eq!(value.get_bit(id), expected);
        }

        // boolから生成
        #[test]
        fn generate_from_bit() {
            // 初期化
            let flags = vec![Some(false), Some(true), None, None];
            let status: usize = 0b0010;

            // テスト
            assert_eq!(status.generate_from_bit(&flags), 0b0110)
        }
    }
}
