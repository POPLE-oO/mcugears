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
    }
    impl<R: Registers> Command<R> for ExampleCommand {
        fn run(&self, registers: &mut R) -> CommandResult {
            match self {
                Self::Add { id_d, id_r } => Self::add(registers, *id_d, *id_r),
            }
        }

        fn is_side_effect(&self) -> bool {
            todo!()
        }
    }

    impl ExampleCommand {
        pub fn add<R: Registers>(
            registers: &mut R,
            id_d: RegisterId,
            id_r: RegisterId,
        ) -> CommandResult {
            let rd = registers.read_register_value(RegisterType::General { id: id_d });
            let rr = registers.read_register_value(RegisterType::General { id: id_r });
            registers.execute_operation(RegisterOperation::Add {
                register_type: RegisterType::General { id: id_d },
                value: rr,
            });

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
    }
}
