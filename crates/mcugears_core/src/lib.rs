#![allow(dead_code)]
use std::iter::Iterator;

// Mcu要素のインポート
pub mod command;
pub mod registers;
use command::*;
use registers::*;

// 既定の型
pub type RegisterId = u8; // レジスタのidを格納するための型
pub type RegisterSize = i64; // レジスタの最大サイズ

// マイコン操作の実体オブジェクト
struct Mcu<R, C>
where
    R: Registers,
    C: Command<R>,
{
    registers: R,     // レジスタの構造体
    commands: Vec<C>, // 命令列
}

// マイコン操作の実装
impl<R, C> Mcu<R, C>
where
    R: Registers,
    C: Command<R>,
{
    // コンストラクタ
    fn new(registers: R, commands: Vec<C>) -> Self {
        Mcu {
            registers,
            commands,
        }
    }

    // 副作用を１つ実行
    fn run_side_effect(&mut self) -> Option<String> {
        // プログラムカウンター取得
        let current_program_coutnter = self.registers.read_program_counter();

        // 命令取得
        let command = &self.commands[current_program_coutnter as usize];
        // 命令実行
        let result = command.run(&mut self.registers);

        // タイマーアップデート
        self.registers.update_timer(result.clocks());

        // プログラムカウンター更新
        let next_program_counter = self
            .registers
            .update_program_counter(result.program_couter_change());

        // 次の命令に副作用があるかで分岐
        // 次の命令取得
        let next_command = &self.commands[next_program_counter as usize];
        match next_command.command_type() {
            // 副作用がないなら
            CommandType::SelfContained => None, // ループ終了
            // 副作用があるなら
            CommandType::SideEffect => Some(result.debug_info()), // 次のループ
        }
    }

    // 副作用をまとめて実行
    fn run_side_effect_batch(&mut self) -> Vec<String> {
        // デバック用文字列を取得
        let mut debug_infos: Vec<String> = Vec::new();
        // 副作用が終わるまでループ
        while let Some(debug_info) = self.run_side_effect() {
            debug_infos.push(debug_info);
        }
        debug_infos
    }
}

// マイコン実行はイテレーションで行う
impl<R, C> Iterator for Mcu<R, C>
where
    R: Registers,
    C: Command<R>,
{
    type Item = String;
    // 次の命令を実行する
    fn next(&mut self) -> Option<Self::Item> {
        // プログラムカウンター取得
        let current_program_coutnter = self.registers.read_program_counter();

        // 命令取得
        let command = &self.commands[current_program_coutnter as usize];
        // 命令実行
        let result = command.run(&mut self.registers);

        // タイマーアップデート
        self.registers.update_timer(result.clocks());

        // プログラムカウンター更新
        let next_program_counter = self
            .registers
            .update_program_counter(result.program_couter_change());

        // 次の命令に副作用があるかで分岐
        // 次の命令取得
        let next_command = &self.commands[next_program_counter as usize];
        match next_command.command_type() {
            // 副作用がないなら
            CommandType::SelfContained => Some(result.debug_info()), // 次のループ
            // 副作用があるなら
            CommandType::SideEffect => None, // ループ終了
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
