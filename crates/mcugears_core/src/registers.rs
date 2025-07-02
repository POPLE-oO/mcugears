// レジスタを表す構造体
trait Registers {
    // 初期化
    fn new() -> Self;
    // 書き込み
    fn write_to(&mut self, register_type: RegisterType, value: usize) -> &mut Self;
    // 読み込み
    fn read_from(&self, register_type: RegisterType) -> usize;
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

#[cfg(test)]
mod registers_tests {
    use super::*;

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

    // レジスタ操作
    #[cfg(test)]
    mod operation {

        use rstest::rstest;

        use super::*;

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
    }
}
