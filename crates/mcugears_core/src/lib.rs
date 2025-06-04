#![allow(dead_code)]
use std::iter::Iterator;

type RegisterNum = u8; // すべてのレジスタが収まる型
type RegisterSize = u64; // レジスタの大きさ

// レジスタ構造体の振る舞い
trait Registers {
    // コンストラクタ
    fn new() -> Self;
    // レジスタの種類、値などを受け取って変更したりする
    fn operate(&mut self, operation: &RegisterOperation) -> &mut Self;
}

// レジスタの種類
enum RegisterKind {
    General { id: RegisterNum }, // 汎用レジスタ
    Uart { id: RegisterNum },    // UARTのステータス
    ProgramCounter,              // プログラムカウンタ
}

// レジスタ操作の種類の列挙型
enum RegisterOperation<'a> {
    Write {
        kind: RegisterKind,
        value: RegisterSize, // 変更する値
    },
    Read {
        kind: RegisterKind,
        result: &'a mut RegisterSize, // 読み取った結果
    },
    None,
}

// 一つの命令(命令の種類のEnum)の振る舞い
trait Command<R: Registers> {
    fn run(&self, registers: &mut R) -> Option<String>;
}

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
}

// マイコン実行はイテレータ操作で行う
impl<R, C> Iterator for Mcu<R, C>
where
    R: Registers,
    C: Command<R>,
{
    type Item = String;
    // 次の命令を実行する
    fn next(&mut self) -> Option<Self::Item> {
        // プログラムカウンター取得
        let mut program_counter: RegisterSize = 0;
        let register_operation = RegisterOperation::Read {
            kind: RegisterKind::ProgramCounter,
            result: &mut program_counter,
        };
        self.registers.operate(&register_operation);

        // プログラムカウンター更新
        // program_counterは更新されない
        let register_operation = RegisterOperation::Write {
            kind: RegisterKind::ProgramCounter,
            value: program_counter + 1,
        };
        self.registers.operate(&register_operation);

        // 命令取得
        let command = &self.commands[program_counter as usize];
        // 命令実行
        command.run(&mut self.registers)
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
    mod enum_registeroperation {
        use super::*;
    }

    #[cfg(test)]
    mod struct_mcu {
        use super::*;
    }
}
