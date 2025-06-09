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
pub struct Mcu<R, C>
where
    R: Registers,
    C: Command<R>,
{
    registers: R,                      // レジスタの構造体
    commands: Vec<C>,                  // 命令列
    elapsed_clocks: Vec<RegisterSize>, // タイマーのカウントアップからの経過時間(プリスケーラ用)
}

// マイコン操作の実装
impl<R, C> Mcu<R, C>
where
    R: Registers,
    C: Command<R>,
{
    // コンストラクタ
    fn new(registers: R, commands: Vec<C>) -> Self {
        let timer_num = registers.get_register_num(&RegisterType::Timer { id: 0 }) as usize;
        Mcu {
            registers,
            commands,
            elapsed_clocks: vec![0; timer_num],
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
        self.registers
            .update_timer(&mut self.elapsed_clocks, result.clocks());

        // プログラムカウンター更新
        let next_program_counter = self
            .registers
            .update_program_counter(result.program_couter_change());

        // 次の命令に副作用があるかで分岐
        // 次の命令取得
        let next_command = &self.commands[next_program_counter as usize];
        if next_command.is_side_effect() {
            // 副作用があるなら
            Some(result.debug_info()) // 次のループ
        } else {
            // 副作用がないなら
            None // ループ終了
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
        self.registers
            .update_timer(&mut self.elapsed_clocks, result.clocks());

        // プログラムカウンター更新
        let next_program_counter = self
            .registers
            .update_program_counter(result.program_couter_change());

        // 次の命令に副作用があるかで分岐
        // 次の命令取得
        let next_command = &self.commands[next_program_counter as usize];
        if next_command.is_side_effect() {
            // 副作用があるなら
            None // ループ終了
        } else {
            // 副作用がないなら
            Some(result.debug_info()) // 次のループ
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
