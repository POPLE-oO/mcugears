// レジスタを表す構造体
trait Registers {
    fn new() -> Self;
}

#[cfg(test)]
pub mod utilities {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub struct ExampleRegisters {
        general: [u8; 32],
        sreg: u8,
        stack_pointer: u16,
        program_counter: u16,
        io: [u8; 256],
    }

    impl Registers for ExampleRegisters {
        fn new() -> Self {
            ExampleRegisters {
                general: [0; 32],
                sreg: 0,
                stack_pointer: 0,
                program_counter: 0,
                io: [0; 256],
            }
        }
    }

    // registersの初期化
    #[cfg(test)]
    mod initialize {
        use super::*;

        #[test]
        fn default() {
            let registers = ExampleRegisters::new();

            assert_eq!(
                registers,
                ExampleRegisters {
                    general: [0; 32],
                    sreg: 0,
                    stack_pointer: 0,
                    program_counter: 0,
                    io: [0; 256],
                }
            )
        }
    }
}

#[cfg(test)]
mod registers_tests {
    use super::utilities::*;
}
