// ルートから読み込み
use crate::*;
use core::fmt::Debug;

// 既定の型
pub type RegisterId = u8; // レジスタのidを格納するための型
pub type RegisterSize = u64; // レジスタの最大サイズ

// レジスタ構造体の振る舞い
pub trait Registers {
    // statusレジスタの種類を表す型
    type StatusType: Debug + Clone + Copy + PartialEq;

    // コンストラクタ
    fn new() -> Self;

    // 値の設定
    fn set_register(&mut self, register_type: RegisterType<Self::StatusType>, value: RegisterSize);

    // 値取得
    fn read_register_value(&self, register_type: RegisterType<Self::StatusType>) -> RegisterSize;

    // レジスタの変更操作を受け取り変更をする
    fn execute_operation(&mut self, operation: RegisterOperation<Self::StatusType>) -> &mut Self {
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
    fn execute_operation_batch(&mut self, operations: &[RegisterOperation<Self::StatusType>]) {
        for operation in operations {
            self.execute_operation(*operation);
        }
    }

    // タイマーを更新
    fn update_timer(&mut self, clocks: RegisterSize) -> &mut Self;

    // プログラムカウンター(命令アドレス)の値を返す
    fn read_program_counter(&self) -> RegisterSize {
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

// レジスタの種類(クエリ)
#[derive(Debug, Clone, Copy)]
pub enum RegisterType<S> {
    General { id: RegisterId },                   // 汎用レジスタ
    Timer { id: RegisterId },                     // タイマー(経過時間)
    ProgramCounter,                               // プログラムカウンタ(命令アドレス)
    Status { status_name: S, index: RegisterId }, // ステータスレジスタ
}

// レジスタ操作の種類の列挙型
#[derive(Debug, Clone, Copy)]
pub enum RegisterOperation<S> {
    //書き込み
    Write {
        register_type: RegisterType<S>, // レジスタ指定
        value: RegisterSize,            // 変更する値
    },
    //加算
    Add {
        register_type: RegisterType<S>,
        value: RegisterSize, // 追加する値
    },
    // 何もしない
    None,
}

#[cfg(test)]
pub mod test_utilities {
    use super::*;

    // レジスタの個数
    pub mod example_registers_max_id {
        pub const GENERAL: usize = 32;
        pub const TIMER: usize = 1;
    }

    // レジスタサイズの型
    pub mod example_registers_size {
        pub type General = u8;
        pub type Timer = u16;
        pub type PrescalerInterval = u16;
        pub type ProgramCounter = u16;
    }
    use example_registers_max_id as register_max_id;
    use example_registers_size as register_size;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ExampleStatusType {
        PrescalerInterval,
    }

    #[derive(Default, Debug, PartialEq)]
    pub struct ExampleRegisters {
        general: [register_size::General; register_max_id::GENERAL],
        timer: [register_size::Timer; register_max_id::TIMER],
        program_counter: register_size::ProgramCounter,
        prescaler_interval: [register_size::PrescalerInterval; register_max_id::TIMER],
    }

    impl Registers for ExampleRegisters {
        type StatusType = ExampleStatusType;

        fn new() -> Self {
            Self::default()
        }

        fn set_register(
            &mut self,
            register_type: RegisterType<Self::StatusType>,
            value: RegisterSize,
        ) {
            match register_type {
                RegisterType::General { id } => {
                    self.general[id as usize] = value as register_size::General
                }
                RegisterType::Timer { id } => {
                    self.timer[id as usize] = value as register_size::Timer
                }
                RegisterType::ProgramCounter => {
                    self.program_counter = value as register_size::ProgramCounter
                }
                RegisterType::Status { status_name, index } => match status_name {
                    ExampleStatusType::PrescalerInterval => {
                        self.prescaler_interval[index as usize] =
                            value as register_size::PrescalerInterval
                    }
                },
            }
        }

        fn read_register_value(
            &self,
            register_type: RegisterType<Self::StatusType>,
        ) -> RegisterSize {
            match register_type {
                RegisterType::General { id } => self.general[id as usize] as RegisterSize,
                RegisterType::Timer { id } => self.timer[id as usize] as RegisterSize,
                RegisterType::ProgramCounter => self.program_counter as RegisterSize,
                RegisterType::Status { status_name, index } => match status_name {
                    ExampleStatusType::PrescalerInterval => {
                        self.prescaler_interval[index as usize] as RegisterSize
                    }
                },
            }
        }

        fn update_timer(&mut self, clocks: RegisterSize) -> &mut Self {
            // プリスケーラ定義(仮)
            let prescalers = [64];

            for i in 0..register_max_id::TIMER {
                // 経過時間追加
                let elapsed = self
                    .execute_operation(RegisterOperation::Add {
                        register_type: RegisterType::Status {
                            status_name: ExampleStatusType::PrescalerInterval,
                            index: i as RegisterId,
                        },
                        value: clocks,
                    })
                    .read_register_value(RegisterType::Status {
                        status_name: ExampleStatusType::PrescalerInterval,
                        index: i as RegisterId,
                    });

                // タイマーアップデート
                self.execute_operation(RegisterOperation::Add {
                    register_type: RegisterType::Timer {
                        id: i as RegisterId,
                    },
                    value: elapsed / prescalers[i], // プリスケーラの閾値を超えたらタイマーをアップデート
                });

                // プリスケーラの起動しない部分は経過時間として保存しておく
                self.execute_operation(RegisterOperation::Write {
                    register_type: RegisterType::Status {
                        status_name: ExampleStatusType::PrescalerInterval,
                        index: i as RegisterId,
                    },
                    value: elapsed % prescalers[i],
                });
            }

            self
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

        // --- set_register, read_register_valueのテスト---
        // register.generalのset,read
        #[test]
        fn test_set_read_register_general() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Status {
                status_name: ExampleStatusType::PrescalerInterval,
                index: 0,
            };
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

        // statusのset,read
        #[test]
        fn test_set_read_register_status() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Status {
                status_name: ExampleStatusType::PrescalerInterval,
                index: 0,
            };
            registers.set_register(register_type, 15);

            assert_eq!(registers.read_register_value(register_type), 15);
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
            registers.set_register(register_type, 65636);

            assert_eq!(registers.read_register_value(register_type), 100);
        }

        // program_counterの切り捨て
        #[test]
        fn test_set_register_truncation_program_counter() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::ProgramCounter;
            registers.set_register(register_type, 67056);

            assert_eq!(registers.read_register_value(register_type), 1520);
        }

        // statusの切り捨て
        #[test]
        fn test_set_register_truncation_status() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Status {
                status_name: ExampleStatusType::PrescalerInterval,
                index: 0,
            };
            registers.set_register(register_type, 65724);

            assert_eq!(registers.read_register_value(register_type), 188);
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

            registers.execute_operation_batch(&[
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

            registers.update_timer(100);

            assert_eq!(registers.read_register_value(register_type), 1);
        }

        // update_timerが何度も実行されたとき
        #[test]
        fn test_update_timer_repeatedly() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };

            for _ in 0..50 {
                registers.update_timer(2);
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

            registers.update_timer(1);

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

            // 実行数
            println!(
                "[NUM]:operations/seconds: {}",
                MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND
            );

            // 適当な値でoperationsを生成
            let operations: Vec<RegisterOperation<ExampleStatusType>> =
                std::iter::repeat_with(|| RegisterOperation::Write {
                    register_type: RegisterType::General {
                        id: rand::random_range(0..32),
                    },
                    value: rand::random_range(0..100),
                })
                .take(1000000)
                .collect();

            // 計測開始
            let start = Instant::now();
            // operationsを実行
            registers.execute_operation_batch(&operations);
            // 計測終了
            let elapsed = start.elapsed();

            // ベンチ結果
            let result = (elapsed.as_millis() as f64 / 1000000.0)
                * (MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND) as f64;
            // 実行結果
            println!("[RESULT]registers: {registers:?}");
            // パフォーマンス
            // (elapsed/10000) * (operations/seconds)で1秒間に行うべき処理にかかった時間を計算
            println!("[PERFORMANCE]execute_operation_batch: {result:?}/1000.00ms",);
            // 1秒に収まっているか
            assert!(result < 1000.0);
        }

        // addのパフォーマンス
        #[test]
        #[ignore]
        fn bench_execute_operation_performance_add() {
            let mut registers = ExampleRegisters::new();

            // 実行数
            println!(
                "[NUM]:operations/seconds: {}",
                MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND
            );

            // 適当な値でoperationsを生成
            let operations: Vec<RegisterOperation<ExampleStatusType>> =
                std::iter::repeat_with(|| RegisterOperation::Add {
                    register_type: RegisterType::General {
                        id: rand::random_range(0..32),
                    },
                    value: rand::random_range(0..100),
                })
                .take(1000000)
                .collect();

            // 計測開始
            let start = Instant::now();
            // operationsを実行
            registers.execute_operation_batch(&operations);
            // 計測終了
            let elapsed = start.elapsed();

            // ベンチ結果
            let result = (elapsed.as_millis() as f64 / 1000000.0)
                * (MAX_CLOCKS_FREQUENCY * OPERATIONS_IN_ONE_COMMAND) as f64;
            // 実行結果
            println!("[RESULT]registers: {registers:?}");
            // パフォーマンス
            // (elapsed/10000) * (operations/seconds)で1秒間に行うべき処理にかかった時間を計算
            println!("[PERFORMANCE]execute_operation_batch: {result:?}/1000.00ms",);
            // 1秒に収まっているか
            assert!(result < 1000.0);
        }
    }
}
