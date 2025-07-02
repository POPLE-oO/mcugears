// レジスタを表す構造体
trait Registers {
    // 初期化
    fn new() -> Self;
    // 書き込み
    fn write_to<V>(&mut self, register_type: RegisterType, value: V) -> &mut Self
    where
        V: Into<u8> + Into<u16>;
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
        fn write_to<V>(&mut self, register_type: RegisterType, value: V) -> &mut Self
        where
            V: Into<u8> + Into<u16>,
        {
            // 書き込み
            match register_type {
                RegisterType::General { id } => self.general[id] = value.into(),
                RegisterType::Status => self.status = value.into(),
                RegisterType::StackPointer => self.stack_pointer = value.into(),
                RegisterType::ProgramCounter => self.program_counter = value.into(),
                RegisterType::Io { id } => self.io[id] = value.into(),
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
            let mut result = ExampleRegisters {
                general: [0; 32],
                status: 0,
                stack_pointer: 0,
                program_counter: 0,
                io: [0; 256],
            };
            result.general[14] = 140;

            // テスト
            assert_eq!(registers, result);
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
    }
}
