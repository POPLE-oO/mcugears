// ルートから読み込み
use crate::*;

// 一つの命令(命令の種類のEnum)の振る舞い
pub trait Command<R: Registers> {
    // コマンドを実行
    fn run(&self, registers: &mut R) -> CommandResult;
    // 現在のコマンドの種類を取得
    fn command_type(&self) -> CommandType;

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
pub enum CommandType {
    SelfContained, // 副作用を含まない。 他に影響しない、されない。
    SideEffect,    // 副作用[IO]
}
pub struct CommandResult {
    debug_info: String,                           // 実行したコマンドの詳細(デバック用)
    clocks: RegisterSize,                         // 実行クロック
    program_counter_change: ProgramCounterChange, // プログラムカウンタ更新情報
}

// プログラムカウンター(命令アドレス)の更新方法
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
    pub fn program_couter_change(&self) -> &ProgramCounterChange {
        &self.program_counter_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExampleCommad::*;

    enum ExampleCommad {
        Add(RegisterId, RegisterId),
    }

    impl<R: Registers> Command<R> for ExampleCommad {
        fn run(&self, registers: &mut R) -> CommandResult {
            match self {
                Self::Add(rd, rr) => Self::add(registers, *rd, *rr),
            }
        }

        fn command_type(&self) -> CommandType {
            match self {
                _ => CommandType::SelfContained,
            }
        }
    }

    impl ExampleCommad {
        pub fn add<R: Registers>(
            registers: &mut R,
            rd: RegisterId,
            rr: RegisterId,
        ) -> CommandResult {
            let mut value = 0;
            registers
                .operate(&RegisterOperation::Read {
                    register_type: RegisterType::General { id: rr },
                    result: &mut value,
                })
                .operate(&RegisterOperation::Add {
                    register_type: RegisterType::General { id: rd },
                    value,
                });

            CommandResult::new("[ADD]: Rd ← Rd + Rr", 1, ProgramCounterChange::Default)
        }
    }

    #[test]
    fn test_command_type_exist() {
        // 存在するか
        let _ = CommandType::SelfContained;
        let _ = CommandType::SideEffect;
    }
}
