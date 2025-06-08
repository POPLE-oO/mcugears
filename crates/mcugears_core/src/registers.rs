// ルートから読み込み
use crate::*;

// レジスタ構造体の振る舞い
pub trait Registers {
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
            register_type: RegisterType::ProgramCounter,
            result: &mut program_counter,
        };
        self.operate(&register_operation);

        // 現在の値を返す
        program_counter
    }
    // プログラムカウンター(命令アドレス)を更新して、更新後の値を返す
    fn update_program_counter(
        &mut self,
        program_couter_change: &ProgramCounterChange,
    ) -> RegisterSize {
        // プログラムカウンター更新

        let register_operation = match program_couter_change {
            // インクリメントで変更(PC←PC+1)
            ProgramCounterChange::Default => RegisterOperation::Add {
                register_type: RegisterType::ProgramCounter,
                value: 1,
            },

            // 相対アドレスで変更
            ProgramCounterChange::Relative(change) => RegisterOperation::Add {
                register_type: RegisterType::ProgramCounter,
                value: *change,
            },

            // 絶対アドレスで変更
            ProgramCounterChange::Absolute(address) => RegisterOperation::Write {
                register_type: RegisterType::ProgramCounter,
                value: *address,
            },
        };
        self.operate(&register_operation);

        // 更新後の値を返す
        self.read_program_counter()
    }
}

// レジスタの種類
pub enum RegisterType {
    General { id: RegisterId }, // 汎用レジスタ
    Timer { id: RegisterId },   // タイマー(経過時間)
    ProgramCounter,             // プログラムカウンタ(命令アドレス)
}

// レジスタ操作の種類の列挙型
pub enum RegisterOperation<'a> {
    Write {
        register_type: RegisterType,
        value: RegisterSize, // 変更する値
    },
    Add {
        register_type: RegisterType,
        value: RegisterSize, // 追加する値
    },
    Read {
        register_type: RegisterType,
        result: &'a mut RegisterSize, // 読み取った結果
    },
    TimerUpdate {
        // すべてのタイマーを指定したクロック数で更新する
        clocks: RegisterSize, // クロック数
    },
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
}
