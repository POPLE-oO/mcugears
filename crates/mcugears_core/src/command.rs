// ルートから読み込み
use crate::*;

// 一つの命令(命令の種類のEnum)の振る舞い
// 命令長の違いでアドレスがずれるものはパース時に吸収する
pub trait Command<R: Registers> {
    fn run(&self, registers: &mut R) -> CommandResult;
    fn command_type(&self) -> CommandType;
}
pub struct CommandResult {
    pub debug_info: String,   // 実行したコマンドの詳細(デバック用)
    pub clocks: RegisterSize, // 実行クロック
}

pub enum CommandType {
    SelfContained, // 副作用を含まない。 他に影響しない、されない。
    SideEffect,    // 副作用[IO]
}
impl CommandResult {
    pub fn new(debug_info: &str, clocks: RegisterSize) -> CommandResult {
        CommandResult {
            debug_info: debug_info.to_string(),
            clocks,
        }
    }
    pub fn debug_info(self) -> String {
        self.debug_info
    }
    pub fn clocks(&self) -> RegisterSize {
        self.clocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
