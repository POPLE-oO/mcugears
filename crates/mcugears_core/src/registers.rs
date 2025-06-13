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
    fn read_register_num(&self, register_type: RegisterType) -> RegisterId;

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
            RegisterOperation::None => {}
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
            ProgramCounterChange::Default => RegisterOperation::Add {
                register_type: RegisterType::ProgramCounter,
                value: 1,
            },
            ProgramCounterChange::Relative(change) => RegisterOperation::Add {
                register_type: RegisterType::ProgramCounter,
                value: change,
            },
            ProgramCounterChange::Absolute(address) => RegisterOperation::Write {
                register_type: RegisterType::ProgramCounter,
                value: address,
            },
            ProgramCounterChange::Jumped => RegisterOperation::None,
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
    //書き込み
    Write {
        register_type: RegisterType, // レジスタ指定
        value: RegisterSize,         // 変更する値
    },
    //加算
    Add {
        register_type: RegisterType,
        value: RegisterSize, // 追加する値
    },
    // 何もしない
    None,
}

#[cfg(test)]
pub mod test_utilities {
    use super::*;

    // レジスタに関する定数
    const MAX_GENERAL_COUNT: usize = 32;
    const MAX_TIMER_COUNT: usize = 1;

    // レジスタサイズの型
    pub type RegisterSizeGeneral = u8;
    pub type RegisterSizeTimer = u8;
    pub type RegisterSizeProgramCounter = u16;

    #[derive(Debug, PartialEq)]
    pub struct ExampleRegisters {
        general: [RegisterSizeGeneral; MAX_GENERAL_COUNT],
        timer: [RegisterSizeTimer; MAX_TIMER_COUNT],
        program_counter: RegisterSizeProgramCounter,
        // prescaler_interval: [RegisterSize; MAX_TIMER_COUNT],
    }

    impl Registers for ExampleRegisters {
        fn new() -> Self {
            Self {
                general: [0; MAX_GENERAL_COUNT],
                timer: [0; MAX_TIMER_COUNT],
                program_counter: 0,
            }
        }

        fn set_register(&mut self, register_type: RegisterType, value: RegisterSize) {
            match register_type {
                RegisterType::General { id } => {
                    self.general[id as usize] = value as RegisterSizeGeneral
                }
                RegisterType::Timer { id } => self.timer[id as usize] = value as RegisterSizeTimer,
                RegisterType::ProgramCounter => {
                    self.program_counter = value as RegisterSizeProgramCounter
                }
            }
        }

        fn read_register_value(&self, register_type: RegisterType) -> RegisterSize {
            match register_type {
                RegisterType::General { id } => self.general[id as usize] as RegisterSize,
                RegisterType::Timer { id } => self.timer[id as usize] as RegisterSize,
                RegisterType::ProgramCounter => self.program_counter as RegisterSize,
            }
        }

        fn read_register_num(&self, register_type: RegisterType) -> RegisterId {
            let count = match register_type {
                RegisterType::General { id: _ } => MAX_GENERAL_COUNT,
                RegisterType::Timer { id: _ } => MAX_TIMER_COUNT,
                RegisterType::ProgramCounter => 1,
            };
            count as RegisterId
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
            assert_eq!(registers.general, [0; MAX_GENERAL_COUNT]);
            assert_eq!(registers.timer, [0; MAX_TIMER_COUNT]);
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
        // ---  read_register_num  ---
        // generalのcount
        #[test]
        fn test_read_register_num_general() {
            let registers = ExampleRegisters::new();
            assert_eq!(
                registers.read_register_num(RegisterType::General { id: 0 }),
                32
            );
        }

        // timerのcount
        #[test]
        fn test_read_register_num_timer() {
            let registers = ExampleRegisters::new();
            assert_eq!(
                registers.read_register_num(RegisterType::Timer { id: 0 }),
                1
            );
        }

        // program_counterのcount
        #[test]
        fn test_read_register_num_program_counter() {
            let registers = ExampleRegisters::new();
            assert_eq!(registers.read_register_num(RegisterType::ProgramCounter), 1);
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
            for _ in 0..50 {
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

    // --- read_program_counter, update_program_counterのテスト ---
    #[cfg(test)]
    mod test_registers_read_program_counter {
        use super::*;

        // ---  read_program_counterのテスト  ---
        // PCの絶対的なupdateとそのread,
        #[test]
        fn test_read_update_program_counter_absolute() {
            let mut registers = ExampleRegisters::new();
            registers.update_program_counter(ProgramCounterChange::Absolute(121));

            assert_eq!(registers.read_program_counter(), 121);
        }

        // PCのデフォルトのread,update
        #[test]
        fn test_read_update_program_counter_default() {
            let mut registers = ExampleRegisters::new();
            registers.update_program_counter(ProgramCounterChange::Absolute(30));
            registers.update_program_counter(ProgramCounterChange::Default);

            assert_eq!(registers.read_program_counter(), 31);
        }

        // PCの相対的なupdateとそのread,
        #[test]
        fn test_read_update_program_counter_relative() {
            let mut registers = ExampleRegisters::new();
            registers.update_program_counter(ProgramCounterChange::Absolute(30));
            registers.update_program_counter(ProgramCounterChange::Relative(3));

            assert_eq!(registers.read_program_counter(), 33);
        }

        // PCが既に書き換わっている場合
        #[test]
        fn test_read_update_program_counter_jumped() {
            let mut registers = ExampleRegisters::new();
            registers.update_program_counter(ProgramCounterChange::Absolute(12));
            registers.update_program_counter(ProgramCounterChange::Jumped);

            assert_eq!(registers.read_program_counter(), 12);
        }
    }

    // --- パフォーマンス計測 ---
    #[cfg(test)]
    mod benchmarks {
        use super::*;
        use std::time::Duration;
        use std::time::Instant;

        // 対応する最大のクロック周波数[16MHz]
        const MAX_CLOCKS_FREQUENCY: usize = 16e6 as usize;
        // 1commandあたりのoperation数[operations/command]
        const OPERATIONS_IN_ONE_COMMAND: usize = 4;

        // --- execute_opeartionのパフォーマンス計測 ---
        // writeのパフォーマンス
        #[test]
        #[ignore]
        fn bench_execute_operation_performance_write() {
            let mut registers = ExampleRegisters::new();

            // 適当な値でoperationsを生成
            // (周波数)*(1 commandのoperation数)で1秒あたりのoperation数[1operations/s]
            let operations: Vec<RegisterOperation> =
                std::iter::repeat_with(|| RegisterOperation::Write {
                    register_type: RegisterType::General {
                        id: rand::random_range(0..32),
                    },
                    value: rand::random_range(0..100),
                })
                .take(MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND)
                .collect();

            // 計測開始
            let start = Instant::now();
            // operationsを実行
            registers.execute_operation_batch(&operations);
            // 計測終了
            let elapsed = start.elapsed();

            // ベンチ結果
            // 実行数
            println!(
                "[DONE]:operations/seconds: {}",
                MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND
            );
            // 実行結果
            println!("[RESULT]registers: {registers:?}");
            // パフォーマンス
            println!("[PERFORMANCE]execute_operation_batch: {elapsed:?}/1000.00ms");
            // 1秒に収まっているか
            assert!(elapsed < Duration::new(1, 0));
        }

        // addのパフォーマンス
        #[test]
        #[ignore]
        fn bench_execute_operation_performance_add() {
            let mut registers = ExampleRegisters::new();

            // 適当な値でoperationsを生成
            // (周波数)*(1 commandのoperation数)で1秒あたりのoperation数[1operations/s]
            let operations: Vec<RegisterOperation> =
                std::iter::repeat_with(|| RegisterOperation::Add {
                    register_type: RegisterType::General {
                        id: rand::random_range(0..32),
                    },
                    value: rand::random_range(0..100),
                })
                .take(MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND)
                .collect();

            // 計測開始
            let start = Instant::now();
            // operationsを実行
            registers.execute_operation_batch(&operations);
            // 計測終了
            let elapsed = start.elapsed();

            // ベンチ結果
            // 実行数
            println!(
                "[DONE]:operations/seconds: {}",
                MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND
            );
            // 実行結果
            println!("[RESULT]registers: {registers:?}");
            // パフォーマンス
            println!("[PERFORMANCE]execute_operation_batch: {elapsed:?}/1000.00ms");
            // 1秒に収まっているか
            assert!(elapsed < Duration::new(1, 0));
        }
    }
}
