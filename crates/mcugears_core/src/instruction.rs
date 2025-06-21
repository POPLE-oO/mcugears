// ルートから読み込み
use crate::*;
use std::fmt::Debug;

// 一つの命令(命令の種類のEnum)の振る舞い
pub trait Instruction: Copy {
    // 命令を実行
    fn run<R: Registers, D: DataSpace>(
        &self,
        registers: &mut R,
        data_space: &mut D,
    ) -> InstructionResult;

    // 一つの命令から実行、レジスタ更新までの流れ
    fn run_cycle<R: Registers, D: DataSpace>(
        &self,
        registers: &mut R,
        data_space: &mut D,
    ) -> String {
        // 命令実行
        let result = self.run(registers, data_space);

        registers
            // タイマーアップデート
            .update_timer(result.clocks())
            // プログラムカウンター更新
            .update_program_counter(result.program_couter_change());

        // デバックログを返す
        result.debug_info()
    }

    // 現在の命令の種類を取得
    fn is_side_effect(&self) -> bool;

    // パース時に通常よりも長い命令の場合は先頭に通常のInstruction enumを用いる
    // その後の空いたアドレスにはempty_operation()を仕込む
    // 例(基本16bit):
    // 32bit命令[JMP k] とすると
    // vec![JMP(u16)、EMPTY] にパースされる
    // JMP=> /*処理*/              //こちらは通常の命令
    // EMPTY=> self.empty_operation()  //本来 k があるところにはempty_operation
    // →パース時にアドレスがずれてしまうの防ぐため
    fn empty_operation() -> InstructionResult {
        InstructionResult::new(
            "[EMPTY]: This is empty address for instructions longer than the base instruction length",
            0,
            ProgramCounterChange::Default,
        )
    }
    fn nop() -> InstructionResult {
        InstructionResult::new(
            "[NOP]: Single cycle no operation",
            1,
            ProgramCounterChange::Default,
        )
    }
}

// 命令の実行結果
#[derive(Debug, Clone, PartialEq)]
pub struct InstructionResult {
    debug_info: String,                           // 実行した命令の詳細(デバック用)
    clocks: RegisterOperateValue,                 // 実行クロック
    program_counter_change: ProgramCounterChange, // プログラムカウンタ更新情報
}

impl InstructionResult {
    pub fn new(
        debug_info: &str,
        clocks: RegisterOperateValue,
        pc_change: ProgramCounterChange,
    ) -> InstructionResult {
        InstructionResult {
            debug_info: debug_info.to_string(),
            clocks,
            program_counter_change: pc_change,
        }
    }
    pub fn debug_info(self) -> String {
        self.debug_info
    }
    pub fn clocks(&self) -> RegisterOperateValue {
        self.clocks
    }
    pub fn program_couter_change(&self) -> ProgramCounterChange {
        self.program_counter_change
    }
}

#[cfg(test)]
pub mod test_utilities {

    use super::*;
    use crate::{RegisterId, RegisterOperateValue};

    #[derive(Debug, Clone, Copy)]
    pub enum ExampleInstruction {
        Add { id_d: RegisterId, id_r: RegisterId },
        Jmp { val_k: RegisterOperateValue },
        Push { id_d: RegisterId },
        Pop { id_d: RegisterId },
        Nop,
        Empty,
    }

    impl Instruction for ExampleInstruction {
        fn run<R: Registers, D: DataSpace>(
            &self,
            registers: &mut R,
            data_space: &mut D,
        ) -> InstructionResult {
            match self {
                ExampleInstruction::Add { id_d, id_r } => Self::add(registers, *id_d, *id_r),
                ExampleInstruction::Jmp { val_k } => Self::jmp(registers, *val_k),
                ExampleInstruction::Empty => Self::empty_operation(),
                ExampleInstruction::Nop => Self::nop(),
                ExampleInstruction::Push { id_d } => Self::push(registers, data_space, *id_d),
                ExampleInstruction::Pop { id_d } => Self::pop(registers, data_space, *id_d),
            }
        }

        fn is_side_effect(&self) -> bool {
            match self {
                _ => false,
            }
        }
    }

    impl ExampleInstruction {
        fn add<R: Registers>(
            registers: &mut R,
            id_d: RegisterId,
            id_r: RegisterId,
        ) -> InstructionResult {
            // それぞれの値取得
            let rd = registers.read_from(RegisterType::General { id: id_d });
            let rr = registers.read_from(RegisterType::General { id: id_r });

            // add実行
            registers.execute(RegisterOperation::Add {
                register_type: RegisterType::General { id: id_d },
                value: rr,
            });

            // 結果
            let result = registers.read_from(RegisterType::General { id: id_d });
            InstructionResult::new(
                &format!(
                    "[ADD]: Add Rd({}):{} and Rr({}):{}, Result:Rd({}):{}",
                    id_d, rd, id_r, rr, id_d, result
                ),
                1,
                ProgramCounterChange::Default,
            )
        }

        fn jmp<R: Registers>(registers: &mut R, val_k: RegisterOperateValue) -> InstructionResult {
            let start_program_counter = registers.read_program_counter();
            registers.update_program_counter(ProgramCounterChange::Absolute(val_k));
            let end_program_counter = registers.read_program_counter();
            InstructionResult::new(
                &format!(
                    "[JMP]: Jump from:{} to:{}, Result:PC:{}",
                    start_program_counter, val_k, end_program_counter
                ),
                3,
                ProgramCounterChange::Jumped,
            )
        }

        fn push<R: Registers, D: DataSpace>(
            registers: &mut R,
            data_space: &mut D,
            id_d: RegisterId,
        ) -> InstructionResult {
            // スタックへ書き込み
            let rd = registers.read_from(RegisterType::General { id: id_d });
            let address = registers.read_stack_pointer();
            data_space.write_to(DataAddress::Byte { address }, rd);

            // スタックポインター更新
            registers.write_stack_pointer(StackPointerChange::Push);

            // result
            let stack_value = data_space.read_from(DataAddress::Byte { address });
            InstructionResult::new(
                &format!(
                    "[PUSH]: Push Rd({}):{}, Result:Stack({:X}):{}",
                    id_d, rd, address, stack_value
                ),
                2,
                ProgramCounterChange::Default,
            )
        }

        fn pop<R: Registers, D: DataSpace>(
            registers: &mut R,
            data_space: &mut D,
            id_d: RegisterId,
        ) -> InstructionResult {
            // スタックポインター更新
            registers.write_stack_pointer(StackPointerChange::Pop);

            // スタックからの読み込み
            let address = registers.read_stack_pointer();
            let value = data_space.read_from(DataAddress::Byte { address });

            // レジスタへ書き込み
            registers.execute(RegisterOperation::Write {
                register_type: RegisterType::General { id: id_d },
                value,
            });

            // result
            let register_value = registers.read_from(RegisterType::General { id: id_d });
            InstructionResult::new(
                &format!(
                    "[POP]: Pop Stack({:X}):{}, Result:Rd({}):{}",
                    address, value, id_d, register_value
                ),
                2,
                ProgramCounterChange::Default,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utilities::*;
    use super::*;
    use crate::data_space::test_utilities::*;
    use crate::data_space::*;
    use crate::registers::test_utilities::*;
    use crate::registers::*;

    // ---  instructionの実行 ---
    #[cfg(test)]
    mod test_instruction_run {
        use super::*;

        // --- instructionの実行テスト ---
        // addの実行
        #[test]
        fn test_instruction_run_add() {
            let mut registers = ExampleRegisters::new();
            let mut data_space = ExampleDataSpace::new();
            registers
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 14 },
                    value: 33,
                })
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 19 },
                    value: 22,
                });
            let instruction = ExampleInstruction::Add { id_d: 14, id_r: 19 };
            let result = instruction.run(&mut registers, &mut data_space);

            assert_eq!(
                result,
                InstructionResult::new(
                    "[ADD]: Add Rd(14):33 and Rr(19):22, Result:Rd(14):55",
                    1,
                    ProgramCounterChange::Default,
                )
            );
        }

        // jmpの実行
        #[test]
        fn test_instruction_run_jmp() {
            let mut registers = ExampleRegisters::new();
            let mut data_space = ExampleDataSpace::new();
            registers.update_program_counter(ProgramCounterChange::Absolute(15));
            let instruction = ExampleInstruction::Jmp { val_k: 1202 };
            let result = instruction.run(&mut registers, &mut data_space);

            assert_eq!(
                result,
                InstructionResult::new(
                    "[JMP]: Jump from:15 to:1202, Result:PC:1202",
                    3,
                    ProgramCounterChange::Jumped,
                )
            );
        }

        // empty_operationの実行
        #[test]
        fn test_empty_operation() {
            let mut registers = ExampleRegisters::new();
            let mut data_space = ExampleDataSpace::new();
            let instruction = ExampleInstruction::Empty;
            let result = instruction.run(&mut registers, &mut data_space);

            assert_eq!(
                result,
                InstructionResult::new(
                    "[EMPTY]: This is empty address for instructions longer than the base instruction length",
                    0,
                    ProgramCounterChange::Default,
                )
            );
        }

        // nopの実行
        #[test]
        fn test_nop() {
            let mut registers = ExampleRegisters::new();
            let mut data_space = ExampleDataSpace::new();
            let instruction = ExampleInstruction::Nop;
            let result = instruction.run(&mut registers, &mut data_space);

            assert_eq!(
                result,
                InstructionResult::new(
                    "[NOP]: Single cycle no operation",
                    1,
                    ProgramCounterChange::Default,
                )
            );
        }

        // pushの実行
        #[test]
        fn test_push() {
            let mut registers = ExampleRegisters::new();
            registers
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 22 },
                    value: 57,
                })
                .write_stack_pointer(StackPointerChange::Absolute(0x8FF));
            let mut data_space = ExampleDataSpace::new();

            let instruction = ExampleInstruction::Push { id_d: 22 };
            let result = instruction.run(&mut registers, &mut data_space);

            assert_eq!(
                result,
                InstructionResult::new(
                    "[PUSH]: Push Rd(22):57, Result:Stack(8FF):57",
                    2,
                    ProgramCounterChange::Default
                )
            );
            assert_eq!(registers.read_from(RegisterType::StackPointer), 0x8FE);
        }

        // popの実行
        #[test]
        fn test_pop() {
            // 初期設定
            let mut registers = ExampleRegisters::new();
            registers
                .write_stack_pointer(StackPointerChange::Absolute(0x8FF))
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 19 },
                    value: 136,
                })
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 13 },
                    value: 220,
                });
            let mut data_space = ExampleDataSpace::new();
            ExampleInstruction::Push { id_d: 13 }.run(&mut registers, &mut data_space);

            // 命令実行
            let instruction = ExampleInstruction::Pop { id_d: 19 };
            let result = instruction.run(&mut registers, &mut data_space);

            // test
            assert_eq!(
                result,
                InstructionResult::new(
                    "[POP]: Pop Stack(8FF):220, Result:Rd(19):220",
                    2,
                    ProgramCounterChange::Default
                )
            );
            assert_eq!(registers.read_from(RegisterType::StackPointer), 0x8FF);
        }
    }

    // ---  副作用かのチェック  ---
    #[cfg(test)]
    mod test_instruction_is_sideeffect {
        use crate::instruction::{Instruction, test_utilities::ExampleInstruction};

        #[test]
        fn test_is_sideefect_default() {
            let instruction_add = ExampleInstruction::Add { id_d: 3, id_r: 2 };
            let instruction_jmp = ExampleInstruction::Jmp { val_k: 112 };

            assert!(!instruction_add.is_side_effect());
            assert!(!instruction_jmp.is_side_effect());
        }
    }

    // --- run_cycleのテスト ---
    #[cfg(test)]
    mod test_instruction_run_cycle {
        use super::*;

        // --- run_cycleの実行 ---
        #[test]
        fn test_run_cycle() {
            let mut registers = ExampleRegisters::new();
            let mut data_space = ExampleDataSpace::new();
            registers
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 12 },
                    value: 32,
                })
                .execute(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 17 },
                    value: 41,
                })
                .update_program_counter(ProgramCounterChange::Absolute(22))
                .update_timer(63);

            ExampleInstruction::Add { id_d: 12, id_r: 17 }
                .run_cycle(&mut registers, &mut data_space);

            assert_eq!(registers.read_from(RegisterType::General { id: 12 }), 73);
            assert_eq!(registers.read_from(RegisterType::General { id: 17 }), 41);
            assert_eq!(registers.read_from(RegisterType::ProgramCounter), 23);
            assert_eq!(registers.read_from(RegisterType::Timer { id: 0 }), 1);
        }
    }
}
