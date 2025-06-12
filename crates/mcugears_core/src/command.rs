// ルートから読み込み
use crate::*;

// 一つの命令(命令の種類のEnum)の振る舞い
pub trait Command<R: Registers> {
    // コマンドを実行
    fn run(&self, registers: &mut R) -> CommandResult;
    // 現在のコマンドの種類を取得
    fn is_side_effect(&self) -> bool;

    // パース時に通常よりも長い命令の場合は先頭に通常のCommand enumを用いる
    // その後の空いたアドレスにはno_operation()を仕込む
    // 例(基本16bit):
    // 32bit命令[JMP k] とすると
    // vec![JMP(u16)、NOP] にパースされる
    // JMP=> /*処理*/              //こちらは通常の命令
    // NOP=> self.no_operation()  //本来 k があるところにはno_operation
    // →パース時にアドレスがずれてしまうの防ぐため
    fn no_operation(&self) -> CommandResult {
        CommandResult::new(
            "[NOP]: This is a no-operation for instructions longer than the base instruction length",
            0,
            ProgramCounterChange::Default,
        )
    }
}

// 命令の実行結果
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    debug_info: String,                           // 実行したコマンドの詳細(デバック用)
    clocks: RegisterSize,                         // 実行クロック
    program_counter_change: ProgramCounterChange, // プログラムカウンタ更新情報
}

// プログラムカウンター(命令アドレス)の更新方法
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgramCounterChange {
    Default,                // PCをインクリメント(PC←PC+1)
    Absolute(RegisterSize), // 絶対アドレス(直接目標のアドレスへ)
    Relative(RegisterSize), // 相対アドレス(現在のアドレスからの変化量)
    Jumped,                 // ジャンプ済み(更新済み)
}

impl CommandResult {
    pub fn new(
        debug_info: &str,
        clocks: RegisterSize,
        pc_change: ProgramCounterChange,
    ) -> CommandResult {
        CommandResult {
            debug_info: debug_info.to_string(),
            clocks,
            program_counter_change: pc_change,
        }
    }
    pub fn debug_info(self) -> String {
        self.debug_info
    }
    pub fn clocks(&self) -> RegisterSize {
        self.clocks
    }
    pub fn program_couter_change(&self) -> ProgramCounterChange {
        self.program_counter_change
    }
}
#[cfg(test)]
pub mod test_utilities {

    use super::*;
    use crate::{RegisterId, RegisterSize};

    pub enum ExampleCommand {
        Add { id_d: RegisterId, id_r: RegisterId },
        Jmp { val_k: RegisterSize },
    }
    impl<R: Registers> Command<R> for ExampleCommand {
        fn run(&self, registers: &mut R) -> CommandResult {
            match self {
                ExampleCommand::Add { id_d, id_r } => Self::add(registers, *id_d, *id_r),
                ExampleCommand::Jmp { val_k } => Self::jmp(registers, *val_k),
            }
        }

        fn is_side_effect(&self) -> bool {
            todo!()
        }
    }

    impl ExampleCommand {
        fn add<R: Registers>(
            registers: &mut R,
            id_d: RegisterId,
            id_r: RegisterId,
        ) -> CommandResult {
            // それぞれの値取得
            let rd = registers.read_register_value(RegisterType::General { id: id_d });
            let rr = registers.read_register_value(RegisterType::General { id: id_r });

            // add実行
            registers.execute_operation(RegisterOperation::Add {
                register_type: RegisterType::General { id: id_d },
                value: rr,
            });

            // 結果
            let result = registers.read_register_value(RegisterType::General { id: id_d });
            CommandResult::new(
                &format!(
                    "[ADD]: Add Rd({}):{} and Rr({}):{}, Result:Rd({}):{}",
                    id_d, rd, id_r, rr, id_d, result
                ),
                1,
                ProgramCounterChange::Default,
            )
        }

        fn jmp<R: Registers>(registers: &mut R, val_k: RegisterSize) -> CommandResult {
            let start_program_counter = registers.read_program_counter();
            registers.update_program_counter(ProgramCounterChange::Absolute(val_k));
            let end_program_counter = registers.read_program_counter();
            CommandResult::new(
                &format!(
                    "[JMP]: Jump from:{} to:{}, Result:PC:{}",
                    start_program_counter, val_k, end_program_counter
                ),
                3,
                ProgramCounterChange::Jumped,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utilities::*;
    use super::*;
    use crate::registers::test_utilities::*;
    use crate::registers::*;

    // ---  commandの実行 ---
    #[cfg(test)]
    mod test_command_run {
        use super::*;

        // --- commandの実行テスト ---
        // addの実行
        #[test]
        fn test_command_run_add() {
            let mut registers = ExampleRegisters::new();
            registers
                .execute_operation(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 14 },
                    value: 33,
                })
                .execute_operation(RegisterOperation::Write {
                    register_type: RegisterType::General { id: 19 },
                    value: 22,
                });
            let command = ExampleCommand::Add { id_d: 14, id_r: 19 };
            let result = command.run(&mut registers);

            assert_eq!(
                result,
                CommandResult::new(
                    &format!(
                        "[ADD]: Add Rd(14):{} and Rr(19):{}, Result:Rd(14):{}",
                        33, 22, 55
                    ),
                    1,
                    ProgramCounterChange::Default,
                )
            );
        }

        // jmpの実行
        #[test]
        fn test_command_run_jmp() {
            let mut registers = ExampleRegisters::new();
            registers.update_program_counter(ProgramCounterChange::Absolute(15));
            let command = ExampleCommand::Jmp { val_k: 1202 };
            let result = command.run(&mut registers);

            assert_eq!(
                result,
                CommandResult::new(
                    &format!("[JMP]: Jump from:{} to:{}, Result:PC:{}", 15, 1202, 1202),
                    3,
                    ProgramCounterChange::Jumped,
                )
            );
        }
    }
}
