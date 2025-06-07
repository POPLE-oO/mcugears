#![allow(dead_code)]
use std::iter::Iterator;

type RegisterId = u8; // レジスタのidを格納するための型
type RegisterSize = u64; // レジスタの最大サイズ

// レジスタ構造体の振る舞い
trait Registers {
    // コンストラクタ
    fn new() -> Self;
    // レジスタの種類、値などを受け取って変更したりする
    fn operate(&mut self, operation: &RegisterOperation) -> &mut Self;
    // 処理を複数受け取ってイテレータで処理する
    fn operate_batch(&mut self, operations: Vec<RegisterOperation>) {
        for operation in operations {
            self.operate(&operation);
        }
    }
    // タイマーを更新
    fn update_timer(&mut self, clocks: RegisterSize) -> &mut Self {
        let operation = RegisterOperation::TimerUpdate { clocks };

        self.operate(&operation);

        self
    }
    // プログラムカウンター(命令アドレス)の値を返す
    fn read_program_counter(&mut self) -> RegisterSize {
        // 現在のプログラムカウンター取得
        let mut program_counter: RegisterSize = 0;
        let register_operation = RegisterOperation::Read {
            kind: RegisterType::ProgramCounter,
            result: &mut program_counter,
        };
        self.operate(&register_operation);

        // 現在の値を返す
        program_counter
    }
    // プログラムカウンター(命令アドレス)を更新して、更新後の値を返す
    fn update_program_counter(&mut self) -> RegisterSize {
        // プログラムカウンター更新
        // program_counterは更新されない
        let register_operation = RegisterOperation::Add {
            kind: RegisterType::ProgramCounter,
            value: 1,
        };
        self.operate(&register_operation);

        // 更新後の値を返す
        self.read_program_counter()
    }
}

// レジスタの種類
enum RegisterType {
    General { id: RegisterId }, // 汎用レジスタ
    Uart { id: RegisterId },    // UARTのステータス
    Timer { id: RegisterId },   // タイマー(経過時間)
    ProgramCounter,             // プログラムカウンタ(命令アドレス)
}

// レジスタ操作の種類の列挙型
enum RegisterOperation<'a> {
    Write {
        kind: RegisterType,
        value: RegisterSize, // 変更する値
    },
    Add {
        kind: RegisterType,
        value: RegisterSize, // 追加する値
    },
    Read {
        kind: RegisterType,
        result: &'a mut RegisterSize, // 読み取った結果
    },
    TimerUpdate {
        // すべてのタイマーを指定したクロック数で更新する
        clocks: RegisterSize, // クロック数
    },
    None,
}

// 一つの命令(命令の種類のEnum)の振る舞い
trait Command<R: Registers> {
    fn run(&self, registers: &mut R) -> CommandResult;
    fn command_type(&self) -> CommandType;
}
struct CommandResult {
    debug_info: String,
    clocks: RegisterSize,
    command_type: CommandType,
}

enum CommandType {
    SelfContained, // 副作用を含まない。 他に影響しない、されない。
    SideEffect,    // 副作用[IO]
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

    // 副作用を１つ実行
    fn run_side_effect(&mut self) -> Option<String> {
        // プログラムカウンター取得
        let current_program_coutnter = self.registers.read_program_counter();
        // プログラムカウンター取得→更新
        let next_program_counter = self.registers.update_program_counter();

        // 命令取得
        let command = &self.commands[current_program_coutnter as usize];
        // 命令実行
        let result = command.run(&mut self.registers);

        // タイマーアップデート
        self.registers.update_timer(result.clocks);

        // 次の命令に副作用があるかで分岐
        // 次の命令取得
        let next_command = &self.commands[next_program_counter as usize];
        match next_command.command_type() {
            // 副作用がないなら
            CommandType::SelfContained => None, // ループ終了
            // 副作用があるなら
            CommandType::SideEffect => Some(result.debug_info), // 次のループ
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
        // プログラムカウンター更新
        let next_program_counter = self.registers.update_program_counter();

        // 命令取得
        let command = &self.commands[current_program_coutnter as usize];
        // 命令実行
        let result = command.run(&mut self.registers);

        // タイマーアップデート
        self.registers.update_timer(result.clocks);

        // 次の命令に副作用があるかで分岐
        // 次の命令取得
        let next_command = &self.commands[next_program_counter as usize];
        match next_command.command_type() {
            // 副作用がないなら
            CommandType::SelfContained => Some(result.debug_info), // 次のループ
            // 副作用があるなら
            CommandType::SideEffect => None, // ループ終了
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
    mod enum_registeroperation {
        use super::*;
    }

    #[cfg(test)]
    mod struct_mcu {
        use super::*;
    }
}
