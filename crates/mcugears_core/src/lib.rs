#![allow(dead_code)]
use std::iter::Iterator;

// レジスタ構造体の振る舞い
trait Registers {
    // コンストラクタ
    fn new() -> Self;
    // レジスタの種類,id,値などを受け取って変更したりする
    fn operate(&mut self, operation: RegisterOperation) -> &mut Self;
}

// レジスタ操作の種類の列挙型
enum RegisterOperation {
    Set { kind: String, id: u8, value: u8 },
    None,
}

// 一つの命令(命令の種類のEnum)の振る舞い
trait Command<R: Registers> {
    fn run(&self, registers: &mut R);
}

// 命令を連続処理するためのイテレータ
trait CommandsIterator: Iterator {}

// マイコン操作の実体オブジェクト
struct Mcu<R, C>
where
    R: Registers,
    C: CommandsIterator,
{
    registers: R, // レジスタの構造体
    commands: C,  // 命令列
}

impl<R, C> Mcu<R, C>
where
    R: Registers,
    C: CommandsIterator,
{
    // コンストラクタ
    fn new(registers: R, commands: C) -> Self {
        Mcu {
            registers,
            commands,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod trait_registers {
        use super::*;
        struct ExampleRegisters {
            r: [u8; 32],
        }
    }

    #[cfg(test)]
    mod trait_command {
        use super::*;
    }

    #[cfg(test)]
    mod trait_commandsiterator {
        use super::*;
    }

    #[cfg(test)]
    mod enum_registeroperation {
        use super::*;
    }

    #[cfg(test)]
    mod struct_mcu {
        use super::*;
    }
}
