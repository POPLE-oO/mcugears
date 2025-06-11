// ルートから読み込み
use crate::*;

// レジスタ構造体の振る舞い
pub trait Registers {
    // コンストラクタ
    fn new() -> Self;
    // 値の設定
    fn set_register(&mut self, register_type: RegisterType, value: RegisterSize);
    // 値取得
    fn read_register_value(&self, register_type: RegisterType) -> RegisterSize;
    // レジスタの数を取得
    fn read_register_count(&self, register_type: RegisterType) -> RegisterId;

    // レジスタの変更操作を受け取り変更をする
    fn execute_operation(&mut self, operation: RegisterOperation) -> &mut Self {
        match operation {
            RegisterOperation::Write {
                register_type,
                value,
            } => self.set_register(register_type, value),

            RegisterOperation::Add {
                register_type,
                value,
            } => self.set_register(
                register_type,
                self.read_register_value(register_type).wrapping_add(value),
            ),
        };

        self
    }

    // 処理を複数受け取ってイテレータで処理する
    fn execute_operation_batch(&mut self, operations: &Vec<RegisterOperation>) {
        for operation in operations {
            self.execute_operation(*operation);
        }
    }

    // タイマーを更新
    fn update_timer(
        &mut self,
        elapsed_clocks_from_timer_update: &mut Vec<RegisterSize>,
        clocks: RegisterSize,
    ) -> &mut Self;

    // プログラムカウンター(命令アドレス)の値を返す
    fn read_program_counter(&mut self) -> RegisterSize {
        // 現在の値を返す
        self.read_register_value(RegisterType::ProgramCounter)
    }

    // プログラムカウンター(命令アドレス)を更新して、更新後の値を返す
    fn update_program_counter(
        &mut self,
        program_couter_change: ProgramCounterChange,
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
                value: change,
            },

            // 絶対アドレスで変更
            ProgramCounterChange::Absolute(address) => RegisterOperation::Write {
                register_type: RegisterType::ProgramCounter,
                value: address,
            },
        };
        self.execute_operation(register_operation);

        // 更新後の値を返す
        self.read_program_counter()
    }
}

// レジスタの種類
#[derive(Debug, Clone, Copy)]
pub enum RegisterType {
    General { id: RegisterId }, // 汎用レジスタ
    Timer { id: RegisterId },   // タイマー(経過時間)
    ProgramCounter,             // プログラムカウンタ(命令アドレス)
}

// レジスタ操作の種類の列挙型
#[derive(Debug, Clone, Copy)]
pub enum RegisterOperation {
    Write {
        register_type: RegisterType,
        value: RegisterSize, // 変更する値
    },
    Add {
        register_type: RegisterType,
        value: RegisterSize, // 追加する値
    },
}

#[cfg(test)]
pub mod test_utilities {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct ExampleRegisters {
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

        fn set_register(&mut self, register_type: RegisterType, value: RegisterSize) {
            match register_type {
                RegisterType::General { id } => self.general[id as usize] = value as u8,
                RegisterType::Timer { id } => self.timer[id as usize] = value as u8,
                RegisterType::ProgramCounter => self.program_counter = value as u16,
            }
        }

        fn read_register_value(&self, register_type: RegisterType) -> RegisterSize {
            match register_type {
                RegisterType::General { id } => self.general[id as usize] as RegisterSize,
                RegisterType::Timer { id } => self.timer[id as usize] as RegisterSize,
                RegisterType::ProgramCounter => self.program_counter as RegisterSize,
            }
        }

        fn read_register_count(&self, register_type: RegisterType) -> RegisterId {
            match register_type {
                RegisterType::General { id: _ } => 32,
                RegisterType::Timer { id: _ } => 1,
                RegisterType::ProgramCounter => 1,
            }
        }

        fn update_timer(
            &mut self,
            elapsed_clocks_from_timer_update: &mut Vec<RegisterSize>,
            clocks: RegisterSize,
        ) -> &mut Self {
            // プリスケーラ定義(仮)
            let prescalers = [64];

            for (id, elapsed_clocks) in elapsed_clocks_from_timer_update.iter_mut().enumerate() {
                // 経過時間追加
                *elapsed_clocks += clocks;

                // プリスケーラ
                self.execute_operation(RegisterOperation::Add {
                    register_type: RegisterType::Timer {
                        id: id as RegisterId,
                    },
                    value: *elapsed_clocks / prescalers[id], // プリスケーラの閾値を超えたらタイマーをアップデート
                });

                // プリスケーラの起動しない部分は経過時間として保存しておく
                *elapsed_clocks %= prescalers[id];
            }

            self
        }
    }

    // private フィールドにかかわるテスト
    #[cfg(test)]
    mod tests {
        use super::*;
        // new関数
        #[test]
        fn test_new() {
            let registers = ExampleRegisters::new();
            assert_eq!(registers.general, [0; 32]);
            assert_eq!(registers.timer, [0; 1]);
            assert_eq!(registers.program_counter, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utilities::*;
    use super::*;

    // ---  Registersの読み書きテスト  ---
    #[cfg(test)]
    mod test_registers_set_read {
        use super::*;
        // ---  read_register_count  ---
        // generalのcount
        #[test]
        fn test_read_register_count_general() {
            let registers = ExampleRegisters::new();
            assert_eq!(
                registers.read_register_count(RegisterType::General { id: 0 }),
                32
            );
        }

        // timerのcount
        #[test]
        fn test_read_register_count_timer() {
            let registers = ExampleRegisters::new();
            assert_eq!(
                registers.read_register_count(RegisterType::Timer { id: 0 }),
                1
            );
        }

        // program_counterのcount
        #[test]
        fn test_read_register_count_program_counter() {
            let registers = ExampleRegisters::new();
            assert_eq!(
                registers.read_register_count(RegisterType::ProgramCounter),
                1
            );
        }

        // --- set_register, read_register_valueのテスト---
        // register.generalのset,read
        #[test]
        fn test_set_read_register_general() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::General { id: 4 };
            registers.set_register(register_type, 42);

            assert_eq!(registers.read_register_value(register_type), 42);
        }

        // register.timerのset,read
        #[test]
        fn test_set_read_register_timer() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };
            registers.set_register(register_type, 211);

            assert_eq!(registers.read_register_value(register_type), 211);
        }

        // register.program_counterのset,read
        #[test]
        fn test_set_read_register_program_counter() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::ProgramCounter;
            registers.set_register(register_type, 101);

            assert_eq!(registers.read_register_value(register_type), 101);
        }

        // ---  set_registerの切り捨て処理  ---
        // generalの切り捨て
        #[test]
        fn test_set_register_truncation_general() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::General { id: 3 };
            registers.set_register(register_type, 265);

            assert_eq!(registers.read_register_value(register_type), 9);
        }

        // timerの切り捨て
        #[test]
        fn test_set_register_truncation_timer() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };
            registers.set_register(register_type, 5000);

            assert_eq!(registers.read_register_value(register_type), 136);
        }

        // program_counterの切り捨て
        #[test]
        fn test_set_register_truncation_program_counter() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::ProgramCounter;
            registers.set_register(register_type, 67056);

            assert_eq!(registers.read_register_value(register_type), 1520);
        }

        // ---  set_registerへ負の値の代入  ---
        // generalへの負の値代入
        #[test]
        fn test_set_register_negative_value_general() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::General { id: 3 };
            registers.set_register(register_type, -13);

            assert_eq!(registers.read_register_value(register_type), 243);
        }

        // timerへの負の値代入
        #[test]
        fn test_set_register_negative_value_timer() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };
            registers.set_register(register_type, -13);

            assert_eq!(registers.read_register_value(register_type), 243);
        }

        // program_couterへの負の値代入
        #[test]
        fn test_set_register_negative_value_program_counter() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::ProgramCounter;
            registers.set_register(register_type, -13);

            assert_eq!(registers.read_register_value(register_type), 65523);
        }
    }

    // ---  Enum RegisterOperation を使用したレジスタ操作  ---
    #[cfg(test)]
    mod test_registers_execute_operation {
        use super::*;
        // ---  execute_operationのテスト  ---
        // writeをexecute_operationで実行する
        #[test]
        fn test_execute_operation_write() {
            let mut registers = ExampleRegisters::new();

            let operation = RegisterOperation::Write {
                register_type: RegisterType::General { id: 2 },
                value: 5,
            };
            registers.execute_operation(operation);

            assert_eq!(
                registers.read_register_value(RegisterType::General { id: 2 }),
                5
            );
        }

        // addをexecute_operationで実行する
        #[test]
        fn test_execute_operation_add() {
            let mut registers = ExampleRegisters::new();

            let operation = RegisterOperation::Add {
                register_type: RegisterType::General { id: 10 },
                value: 100,
            };
            registers.execute_operation(operation);

            assert_eq!(
                registers.read_register_value(RegisterType::General { id: 10 }),
                100
            );
        }

        // --- execute_operation_batchのテスト  ---
        // execute_operation_batch実行
        #[test]
        fn test_execute_operation_batch() {
            let mut registers = ExampleRegisters::new();

            let register_type = RegisterType::General { id: 15 };

            registers.execute_operation_batch(&vec![
                RegisterOperation::Write {
                    register_type,
                    value: 12,
                },
                RegisterOperation::Add {
                    register_type,
                    value: 120,
                },
            ]);
            assert_eq!(registers.read_register_value(register_type), 132);
        }
    }

    // --- update_timer ---
    #[cfg(test)]
    mod test_registers_update_timer {
        use super::*;

        // ---  update_timerのテスト  ---
        // update_timerの実行
        #[test]
        fn test_update_timer() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };

            let mut elapsed_clocks_from_timer_update = vec![0];
            registers.update_timer(&mut elapsed_clocks_from_timer_update, 100);

            assert_eq!(registers.read_register_value(register_type), 1);
        }

        // update_timerが何度も実行されたとき
        #[test]
        fn test_update_timer_repeatedly() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };

            let mut elapsed_clocks_from_timer_update = vec![0];
            for i in 0..50 {
                registers.update_timer(&mut elapsed_clocks_from_timer_update, 2);
            }
            assert_eq!(registers.read_register_value(register_type), 1);
        }

        // ---  elapsed_clocks_from_timer_updateとprescalerの動作確認 ---
        // prescalerが起動しないとき
        // (elapsed_clocks_from_timer_update < prescalerの時)
        #[test]
        fn test_update_timer_prescaler_not_activated() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };

            let mut elapsed_clocks_from_timer_update = vec![0];
            registers.update_timer(&mut elapsed_clocks_from_timer_update, 1);

            assert_eq!(registers.read_register_value(register_type), 0);
        }
    }

    // --- read_program_counter ---
    // --- update_program_counter ---
    // --- read_register_count ---
    // --- エッジケース ---
    // --- 無効なID ---
    // --- パフォーマンス計測 ---
}
