// レジスタを表す構造体
trait Registers {
    // 初期化
    fn new() -> Self;
    // 書き込み
    fn write_to<V>(&mut self, register_type: RegisterType, value: V) -> &mut Self
    where
        V: Into<u8> + Into<u16>;
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
    #[derive(Clone, Debug, PartialEq)]
    pub struct ExampleRegisters {
        general: [u8; 32],
        status: u8,
        stack_pointer: u16,
        program_counter: u16,
        io: [u8; 256],
    }

    impl Registers for ExampleRegisters {
        fn new() -> Self {
            ExampleRegisters {
                general: [0; 32],
                status: 0,
                stack_pointer: 0,
                program_counter: 0,
                io: [0; 256],
            }
        }

        fn write_to<V>(&mut self, register_type: RegisterType, value: V) -> &mut Self
        where
            V: Into<u8> + Into<u16>,
        {
            match register_type {
                RegisterType::General { id } => self.general[id] = Into::into(value),
                RegisterType::Status => self.status = Into::into(value),
                RegisterType::StackPointer => self.stack_pointer = Into::into(value),
                RegisterType::ProgramCounter => self.program_counter = Into::into(value),
                RegisterType::Io { id } => self.io[id] = Into::into(value),
            }

            self
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

            // 操作実行
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
    }
}
