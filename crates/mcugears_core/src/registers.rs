// ルートから読み込み
use crate::*;

// レジスタ構造体の振る舞い
pub trait Registers {
    // コンストラクタ
    fn new() -> Self;
    // 可変参照取得
    fn set_register(&mut self, register_type: &RegisterType, value: RegisterSize);
    // 値取得
    fn get_register(&self, register_type: &RegisterType) -> RegisterSize;
    // レジスタの数を取得
    fn get_register_num(&self, register_type: &RegisterType) -> RegisterId;

    // レジスタの種類、値などを受け取って変更したりする
    fn operate(&mut self, operation: &mut RegisterOperation) -> &mut Self {
        match operation {
            RegisterOperation::Write {
                register_type,
                value,
            } => self.set_register(register_type, *value),

            RegisterOperation::Add {
                register_type,
                value,
            } => self.set_register(register_type, self.get_register(register_type) + *value),

            RegisterOperation::Read {
                register_type,
                result,
            } => **result = self.get_register(register_type),
        };

        self
    }
    // 処理を複数受け取ってイテレータで処理する
    fn operate_batch(&mut self, operations: Vec<&mut RegisterOperation>) {
        for operation in operations {
            self.operate(operation);
        }
    }

    // タイマーを更新
    fn update_timer(
        &mut self,
        elapsed_clocks: &mut Vec<RegisterSize>,
        clocks: RegisterSize,
    ) -> &mut Self;

    // プログラムカウンター(命令アドレス)の値を返す
    fn read_program_counter(&mut self) -> RegisterSize {
        // 現在のプログラムカウンター取得
        let mut program_counter: RegisterSize = 0;
        let register_operation = &mut RegisterOperation::Read {
            register_type: RegisterType::ProgramCounter,
            result: &mut program_counter,
        };
        self.operate(register_operation);

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
            ProgramCounterChange::Default => &mut RegisterOperation::Add {
                register_type: RegisterType::ProgramCounter,
                value: 1,
            },

            // 相対アドレスで変更
            ProgramCounterChange::Relative(change) => &mut RegisterOperation::Add {
                register_type: RegisterType::ProgramCounter,
                value: *change,
            },

            // 絶対アドレスで変更
            ProgramCounterChange::Absolute(address) => &mut RegisterOperation::Write {
                register_type: RegisterType::ProgramCounter,
                value: *address,
            },
        };
        self.operate(register_operation);

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct ExampleRegisters {
        general: [u8; 32],
        timer: [u8; 1],
        program_counter: u16,
    }
    impl Registers for ExampleRegisters {
        fn new() -> Self {
            Self {
                general: [0; 32],
                timer: [0; 1],
                program_counter: 0,
            }
        }

        fn set_register(&mut self, register_type: &RegisterType, value: RegisterSize) {
            match register_type {
                RegisterType::General { id } => self.general[*id as usize] = value as u8,
                RegisterType::Timer { id } => self.timer[*id as usize] = value as u8,
                RegisterType::ProgramCounter => self.program_counter = value as u16,
            }
        }

        fn get_register(&self, register_type: &RegisterType) -> RegisterSize {
            match register_type {
                RegisterType::General { id } => self.general[*id as usize] as RegisterSize,
                RegisterType::Timer { id } => self.timer[*id as usize] as RegisterSize,
                RegisterType::ProgramCounter => self.program_counter as RegisterSize,
            }
        }

        fn get_register_num(&self, register_type: &RegisterType) -> RegisterId {
            match register_type {
                RegisterType::General { id: _ } => 32,
                RegisterType::Timer { id: _ } => 1,
                RegisterType::ProgramCounter => 1,
            }
        }

        fn update_timer(
            &mut self,
            elapsed_clocks: &mut Vec<RegisterSize>,
            clocks: RegisterSize,
        ) -> &mut Self {
            // プリスケーラ定義(仮)
            let prescalers = [64];

            for (id, elapsed_clocks_item) in elapsed_clocks.iter_mut().enumerate() {
                // 経過時間追加
                *elapsed_clocks_item += clocks;

                // プリスケーラ
                self.operate(&mut RegisterOperation::Add {
                    register_type: RegisterType::Timer {
                        id: id as RegisterId,
                    },
                    value: *elapsed_clocks_item / prescalers[id], // プリスケーラの閾値を超えたらタイマーをカウントアップ
                });

                // プリスケーラの起動しない部分は経過時間として保存しておく
                *elapsed_clocks_item %= prescalers[id];
            }

            self
        }
    }

    //new
    #[test]
    fn test_new() {
        let registers = ExampleRegisters::new();
        assert_eq!(registers.general, [0; 32]);
        assert_eq!(registers.timer, [0; 1]);
        assert_eq!(registers.program_counter, 0);
    }

    // set_register, get_register
    #[test]
    fn test_set_get_register_general() {
        let mut registers = ExampleRegisters::new();
        let register_type = &RegisterType::General { id: 4 };
        registers.set_register(register_type, 42);

        assert_eq!(registers.get_register(register_type), 42);
    }

    #[test]
    fn test_set_get_register_timer() {
        let mut registers = ExampleRegisters::new();
        let register_type = &RegisterType::Timer { id: 0 };
        registers.set_register(register_type, 211);

        assert_eq!(registers.get_register(register_type), 211);
    }

    #[test]
    fn test_set_get_register_program_counter() {
        let mut registers = ExampleRegisters::new();
        let register_type = &RegisterType::ProgramCounter;
        registers.set_register(register_type, 101);

        assert_eq!(registers.get_register(register_type), 101);
    }

    // set_register, get_registerの切り捨て処理
    #[test]
    fn test_set_get_register_truncation_general() {}

    // operate
    // operate_batch
    // update_timer
    // read_program_counter
    // update_program_counter
    // get_register_num
    // エッジケース
    // 無効なID
    // パフォーマンス計測
}
