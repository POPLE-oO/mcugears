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
pub struct Mcu<R, I, D>
where
    R: Registers,
    I: Instruction,
    D: DataSpace,
{
    registers: R,         // レジスタの構造体
    instructions: Vec<I>, // 命令列
    data_space: D,        // RAM(stack,heap)
}

// マイコン操作の実装
impl<R, I, D> Mcu<R, I, D>
where
    R: Registers,
    I: Instruction,
    D: DataSpace,
{
    // コンストラクタ
    pub fn new(instructions: Vec<I>, registers: R, data_space: D) -> Self {
        Mcu {
            instructions,
            registers,
            data_space,
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
            Some(instruction.run_cycle(&mut self.registers, &mut self.data_space))
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
            Some(instruction.run_cycle(&mut self.registers, &mut self.data_space))
        } else {
            // 副作用がないなら
            None
        }
    }

    // 副作用以外を実行するイテレータに変換
    #[allow(clippy::wrong_self_convention)] // 本体を更新するためなので&mutでとる必要がある
    fn to_pure_iter<'a>(&'a mut self) -> PureInstructionIterator<'a, R, I, D> {
        PureInstructionIterator { mcu: self }
    }

    // 副作用以外を実行するイテレータに変換
    #[allow(clippy::wrong_self_convention)]
    fn to_side_effect_iter<'a>(&'a mut self) -> SideEffectInstructionIterator<'a, R, I, D> {
        SideEffectInstructionIterator { mcu: self }
    }
}

pub struct PureInstructionIterator<'a, R, I, D>
where
    R: Registers + 'a,
    I: Instruction + 'a,
    D: DataSpace + 'a,
{
    mcu: &'a mut Mcu<R, I, D>, // Mcuの参照
}

impl<'a, R, I, D> Iterator for PureInstructionIterator<'a, R, I, D>
where
    R: Registers,
    I: Instruction,
    D: DataSpace,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.mcu.next_pure()
    }
}

pub struct SideEffectInstructionIterator<'a, R, I, D>
where
    R: Registers + 'a,
    I: Instruction + 'a,
    D: DataSpace + 'a,
{
    mcu: &'a mut Mcu<R, I, D>, // Mcuの参照
}

impl<'a, R, I, D> Iterator for SideEffectInstructionIterator<'a, R, I, D>
where
    R: Registers,
    I: Instruction,
    D: DataSpace,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.mcu.next_side_effect()
    }
}

#[cfg(test)]
mod tests {}
