#![allow(dead_code)]
use std::iter::Iterator;

// Mcu要素のインポート
pub mod data_space;
pub mod instruction;
pub mod registers;
use data_space::*;
use instruction::*;
use registers::*;

// マイコン操作の実体オブジェクト
#[derive(Debug)]
pub struct Mcu<R, C>
where
    R: Registers,
    C: Instruction,
{
    registers: R,         // レジスタの構造体
    instructions: Vec<C>, // 命令列
}

// マイコン操作の実装
impl<R, C> Mcu<R, C>
where
    R: Registers,
    C: Instruction,
{
    // コンストラクタ
    pub fn new(registers: R, instructions: Vec<C>) -> Self {
        Mcu {
            registers,
            instructions,
        }
    }

    // 副作用じゃないなら命令を一つ実行
    pub fn next_pure(&mut self) -> Option<String> {
        // プログラムカウンター取得
        let current_program_coutnter = self.registers.read_program_counter();
        // 命令取得
        let instruction = self.instructions[current_program_coutnter as usize];

        if !instruction.is_side_effect() {
            // 副作用がないなら
            Some(instruction.run_cycle(&mut self.registers))
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
        let instruction = self.instructions[current_program_coutnter as usize];

        if instruction.is_side_effect() {
            // 副作用があるなら
            Some(instruction.run_cycle(&mut self.registers))
        } else {
            // 副作用がないなら
            None
        }
    }

    // 副作用以外を実行するイテレータに変換
    #[allow(clippy::wrong_self_convention)] // 本体を更新するためなので&mutでとる必要がある
    fn to_pure_iter<'a>(&'a mut self) -> PureInstructionIterator<'a, R, C> {
        PureInstructionIterator { mcu: self }
    }

    // 副作用以外を実行するイテレータに変換
    #[allow(clippy::wrong_self_convention)]
    fn to_side_effect_iter<'a>(&'a mut self) -> SideEffectInstructionIterator<'a, R, C> {
        SideEffectInstructionIterator { mcu: self }
    }
}

pub struct PureInstructionIterator<'a, R, C>
where
    R: Registers + 'a,
    C: Instruction + 'a,
{
    mcu: &'a mut Mcu<R, C>, // Mcuの参照
}

impl<'a, R: Registers, C: Instruction> Iterator for PureInstructionIterator<'a, R, C> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.mcu.next_pure()
    }
}

pub struct SideEffectInstructionIterator<'a, R, C>
where
    R: Registers + 'a,
    C: Instruction + 'a,
{
    mcu: &'a mut Mcu<R, C>, // Mcuの参照
}

impl<'a, R: Registers, C: Instruction> Iterator for SideEffectInstructionIterator<'a, R, C> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.mcu.next_side_effect()
    }
}

#[cfg(test)]
mod tests {}
