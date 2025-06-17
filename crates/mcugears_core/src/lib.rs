#![allow(dead_code)]
use std::iter::Iterator;

// Mcu要素のインポート
pub mod command;
pub mod data_space;
pub mod registers;
use command::*;
use data_space::*;
use registers::*;

// マイコン操作の実体オブジェクト
#[derive(Debug)]
pub struct Mcu<R, C>
where
    R: Registers,
    C: Command,
{
    registers: R,     // レジスタの構造体
    commands: Vec<C>, // 命令列
}

// マイコン操作の実装
impl<R, C> Mcu<R, C>
where
    R: Registers,
    C: Command,
{
    // コンストラクタ
    pub fn new(registers: R, commands: Vec<C>) -> Self {
        Mcu {
            registers,
            commands,
        }
    }

    // 副作用じゃないなら命令を一つ実行
    pub fn next_pure(&mut self) -> Option<String> {
        // プログラムカウンター取得
        let current_program_coutnter = self.registers.read_program_counter();
        // 命令取得
        let command = self.commands[current_program_coutnter as usize];

        if !command.is_side_effect() {
            // 副作用がないなら
            Some(command.flow_command(&mut self.registers))
        } else {
            // 副作用があるなら
            None
        }
    }

    // 副作用なら１つ実行
    pub fn next_side_effect(&mut self) -> Option<String> {
        // プログラムカウンター取得
        let current_program_coutnter = self.registers.read_program_counter();
        // 命令取得
        let command = self.commands[current_program_coutnter as usize];

        if command.is_side_effect() {
            // 副作用があるなら
            Some(command.flow_command(&mut self.registers))
        } else {
            // 副作用がないなら
            None
        }
    }

    // 副作用以外を実行するイテレータに変換
    fn to_pure_iter<'a>(&'a mut self) -> PureCommandIterator<'a, R, C> {
        PureCommandIterator { mcu: self }
    }

    // 副作用以外を実行するイテレータに変換
    fn to_side_effect_iter<'a>(&'a mut self) -> PureCommandIterator<'a, R, C> {
        PureCommandIterator { mcu: self }
    }
}

pub struct PureCommandIterator<'a, R, C>
where
    R: Registers + 'a,
    C: Command + 'a,
{
    mcu: &'a mut Mcu<R, C>, // レジスタの構造体
}

impl<'a, R: Registers, C: Command> Iterator for PureCommandIterator<'a, R, C> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.mcu.next_pure()
    }
}

pub struct SideEffectCommandIterator<'a, R, C>
where
    R: Registers + 'a,
    C: Command + 'a,
{
    mcu: &'a mut Mcu<R, C>, // レジスタの構造体
}

impl<'a, R: Registers, C: Command> Iterator for SideEffectCommandIterator<'a, R, C> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.mcu.next_side_effect()
    }
}

#[cfg(test)]
mod tests {}
