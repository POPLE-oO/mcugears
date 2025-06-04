#![allow(dead_code)]
use std::iter::Iterator;

// レジスタ構造体の振る舞い
trait Registers {
    // コンストラクタ
    fn new() -> Self;
    // レジスタの種類、値などを受け取って変更したりする
    fn operate(&mut self, operation: &RegisterOperation) -> &mut Self;
}

// レジスタの種類
enum RegisterKind {
    General { id: u8 }, // 汎用レジスタ
    Uart { id: u8 },    // UARTのステータス
    Counter,
}

// レジスタ操作の種類の列挙型
enum RegisterOperation<'a> {
    Set {
        kind: RegisterKind,
        value: u8, // 変更する値
    },
    Read {
        kind: RegisterKind,
        result: &'a mut u8, // 読み取った結果
    },
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

// マイコン操作の実装
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
