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
    fn write_to(&mut self, register_type: RegisterType<Self::StatusType>, value: RegisterSize);

    // 値取得
    fn read_from(&self, register_type: RegisterType<Self::StatusType>) -> RegisterSize;

    // レジスタの変更操作を受け取り変更をする
    fn execute(&mut self, operation: RegisterOperation<Self::StatusType>) -> &mut Self {
        match operation {
            RegisterOperation::Write {
                register_type,
                value,
            } => self.write_to(register_type, value),
            RegisterOperation::Add {
                register_type,
                value,
            } => self.write_to(
                register_type,
                self.read_from(register_type).wrapping_add(value),
            ),
            RegisterOperation::None => {}
        };

        self
    }

    // 処理を複数受け取ってイテレータで処理する
    fn execute_batch(&mut self, operations: &[RegisterOperation<Self::StatusType>]) {
        for operation in operations {
            self.execute(*operation);
        }
    }

    // タイマーを更新
    fn update_timer(&mut self, clocks: RegisterSize) -> &mut Self;

    // プログラムカウンター(命令アドレス)の値を返す
    fn read_program_counter(&self) -> RegisterSize {
        // 現在の値を返す
        self.read_from(RegisterType::ProgramCounter)
    }

    // プログラムカウンター(命令アドレス)を更新して、更新後の値を返す
    fn update_program_counter(&mut self, program_couter_change: ProgramCounterChange) -> &mut Self {
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
        self.execute(register_operation);

        self
    }
}

// レジスタの種類(クエリ)
#[derive(Debug, Clone, Copy)]
pub enum RegisterType<S> {
    General { id: RegisterId },                   // 汎用レジスタ
    Timer { id: RegisterId },                     // タイマー(経過時間)
    ProgramCounter,                               // プログラムカウンタ(命令アドレス)
    StackPointer,                                 // スタックポインター
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
        pub const DEFAULT: usize = 1;
    }

    // レジスタサイズの型
    pub mod example_registers_size {
        pub type General = u8;
        pub type Timer = u16;
        pub type ProgramCounter = u16;
        pub type StackPointer = u16;
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
        stack_pointer: register_size::StackPointer,
        other_status: ExampleStatusRegisters,
    }

    #[derive(Default, Debug, PartialEq)]
    pub struct ExampleStatusRegisters {
        prescaler_interval: [register_size::Timer; register_max_id::TIMER],
    }

    impl Registers for ExampleRegisters {
        type StatusType = ExampleStatusType;

        fn new() -> Self {
            Self::default()
        }

        fn write_to(&mut self, register_type: RegisterType<Self::StatusType>, value: RegisterSize) {
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
                        self.other_status.prescaler_interval[index as usize] =
                            value as register_size::Timer
                    }
                },
                RegisterType::StackPointer => {
                    self.stack_pointer = value as register_size::StackPointer
                }
            }
        }

        fn read_from(&self, register_type: RegisterType<Self::StatusType>) -> RegisterSize {
            match register_type {
                RegisterType::General { id } => self.general[id as usize] as RegisterSize,
                RegisterType::Timer { id } => self.timer[id as usize] as RegisterSize,
                RegisterType::ProgramCounter => self.program_counter as RegisterSize,
                RegisterType::Status { status_name, index } => match status_name {
                    ExampleStatusType::PrescalerInterval => {
                        self.other_status.prescaler_interval[index as usize] as RegisterSize
                    }
                },
                RegisterType::StackPointer => self.stack_pointer as RegisterSize,
            }
        }

        fn update_timer(&mut self, clocks: RegisterSize) -> &mut Self {
            // プリスケーラ定義(仮)
            let prescalers = [64];

            for i in 0..register_max_id::TIMER {
                // 経過時間追加
                let elapsed = self
                    .execute(RegisterOperation::Add {
                        register_type: RegisterType::Status {
                            status_name: ExampleStatusType::PrescalerInterval,
                            index: i as RegisterId,
                        },
                        value: clocks,
                    })
                    .read_from(RegisterType::Status {
                        status_name: ExampleStatusType::PrescalerInterval,
                        index: i as RegisterId,
                    });

                // タイマーアップデート
                self.execute(RegisterOperation::Add {
                    register_type: RegisterType::Timer {
                        id: i as RegisterId,
                    },
                    value: elapsed / prescalers[i], // プリスケーラの閾値を超えたらタイマーをアップデート
                });

                // プリスケーラの起動しない部分は経過時間として保存しておく
                self.execute(RegisterOperation::Write {
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
    use rstest::rstest;

    // ---  Registersの読み書きテスト  ---
    #[cfg(test)]
    mod test_registers_write_read {
        use super::*;

        // --- write_to, read_from のテスト---
        #[rstest]
        #[case(RegisterType::General { id: 0 }, 42, 42)]
        #[case(RegisterType::Timer { id: 0 }, 211, 211)]
        #[case(RegisterType::ProgramCounter, 101, 101)]
        #[case(RegisterType::Status { status_name: ExampleStatusType::PrescalerInterval, index: 0 }, 15, 15)]
        #[case(RegisterType::StackPointer, 22, 22)]
        fn test_write_read_register(
            #[case] register_type: RegisterType<ExampleStatusType>,
            #[case] value: RegisterSize,
            #[case] expected_value: RegisterSize,
        ) {
            let mut registers = ExampleRegisters::new();
            registers.write_to(register_type, value);
            assert_eq!(registers.read_from(register_type), expected_value);
        }

        // ---  write_toの切り捨て処理  ---
        #[rstest]
        #[case(RegisterType::General { id: 3 }, 265, 9)]
        #[case(RegisterType::Timer { id: 0 }, 65636, 100) ]
        #[case(RegisterType::ProgramCounter, 67056, 1520)]
        #[case(RegisterType::Status { status_name: ExampleStatusType::PrescalerInterval, index: 0 }, 65724, 188)]
        #[case(RegisterType::StackPointer, 74223, 8687)]
        fn test_set_register_truncation(
            #[case] register_type: RegisterType<ExampleStatusType>,
            #[case] value: RegisterSize,
            #[case] expected_truncated_value: RegisterSize,
        ) {
            let mut registers = ExampleRegisters::new();
            registers.write_to(register_type, value);
            assert_eq!(registers.read_from(register_type), expected_truncated_value);
        }
    }

    // ---  Enum RegisterOperation を使用したレジスタ操作  ---
    #[cfg(test)]
    mod test_registers_execute {
        use super::*;
        // ---  executeのテスト  ---
        #[rstest]
        #[case(RegisterOperation::Write{register_type:RegisterType::General{id:2},value:5},RegisterType::General { id: 2 },5)]
        #[case(RegisterOperation::Add{register_type:RegisterType::General{id:21},value:100},RegisterType::General { id: 21 },130)]
        #[case(RegisterOperation::None,RegisterType::General { id: 11 },30)]
        fn test_execute_operation(
            #[case] operation: RegisterOperation<ExampleStatusType>,
            #[case] register_type: RegisterType<ExampleStatusType>,
            #[case] expected_value: RegisterSize,
        ) {
            let mut registers = ExampleRegisters::new();
            registers
                .execute(RegisterOperation::Write {
                    register_type,
                    value: 30,
                })
                .execute(operation);
            assert_eq!(registers.read_from(register_type), expected_value);
        }

        // --- executeの切り捨て処理
        #[rstest]
        #[case(RegisterOperation::Write{register_type:RegisterType::General{id:2},value:272},RegisterType::General { id: 2 },16)]
        #[case(RegisterOperation::Add{register_type:RegisterType::General{id:21},value:300},RegisterType::General { id: 21 },74)]
        #[case(RegisterOperation::None,RegisterType::General { id: 11 },30)]
        fn test_execute_operation_truncation(
            #[case] operation: RegisterOperation<ExampleStatusType>,
            #[case] register_type: RegisterType<ExampleStatusType>,
            #[case] expected_value: RegisterSize,
        ) {
            let mut registers = ExampleRegisters::new();
            registers
                .execute(RegisterOperation::Write {
                    register_type,
                    value: 30,
                })
                .execute(operation);
            assert_eq!(registers.read_from(register_type), expected_value);
        }

        // --- execute_operationのテスト  ---
        // execute_batch実行
        #[test]
        fn test_execute_operation_batch() {
            let mut registers = ExampleRegisters::new();

            let register_type = RegisterType::General { id: 15 };

            registers.execute_batch(&[
                RegisterOperation::Write {
                    register_type,
                    value: 12,
                },
                RegisterOperation::Add {
                    register_type,
                    value: 120,
                },
                RegisterOperation::None,
            ]);
            assert_eq!(registers.read_from(register_type), 132);
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

            registers.update_timer(150);

            assert_eq!(registers.read_from(register_type), 2);
        }

        // --- prescalerの動作確認 ---
        // update_timerが何度も実行されたとき
        #[test]
        fn test_update_timer_repeatedly() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };

            for _ in 0..50 {
                registers.update_timer(2);
            }
            assert_eq!(registers.read_from(register_type), 1);
        }

        // prescalerが起動しないとき
        // (経過時間 < prescalerの時)
        #[test]
        fn test_update_timer_prescaler_not_activated() {
            let mut registers = ExampleRegisters::new();
            let register_type = RegisterType::Timer { id: 0 };

            registers.update_timer(1);

            assert_eq!(registers.read_from(register_type), 0);
        }
    }

    // --- read_program_counter, update_program_counterのテスト ---
    #[cfg(test)]
    mod test_registers_read_program_counter {
        use super::*;

        // ---  read_program_counterのテスト  ---
        #[rstest]
        #[case(ProgramCounterChange::Absolute(121), 121)]
        #[case(ProgramCounterChange::Default, 21)]
        #[case(ProgramCounterChange::Relative(30), 50)]
        #[case(ProgramCounterChange::Jumped, 20)]
        fn test_read_update_program_counter(
            #[case] program_counter_change: ProgramCounterChange,
            #[case] expected_value: RegisterSize,
        ) {
            let mut registers = ExampleRegisters::new();

            registers.update_program_counter(ProgramCounterChange::Absolute(20));
            registers.update_program_counter(program_counter_change);

            assert_eq!(registers.read_program_counter(), expected_value);
        }
    }

    // --- パフォーマンス計測 ---
    #[cfg(test)]
    mod benchmarks {
        use super::*;
        use std::time::Instant;

        // 対応する最大のクロック周波数[16MHz]
        const MAX_CLOCKS_FREQUENCY: usize = 16e6 as usize;
        // 1instructionあたりのoperation数[operations/instruction]
        const OPERATIONS_IN_ONE_COMMAND: usize = 4;

        // --- executeのパフォーマンス計測 ---
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
            registers.execute_batch(&operations);
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
            registers.execute_batch(&operations);
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
