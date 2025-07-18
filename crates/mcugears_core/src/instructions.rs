use crate::{
    registers::{RegisterUpdate, Registers},
    user_ram::UserRam,
};

// 命令のメソッド
pub trait Instruction {
    fn run<R: Registers, U: UserRam>(&self, registers: &mut R, user_ram: &mut U) -> RegisterUpdate;
}

#[cfg(test)]
pub mod instructions_tests {
    use super::*;
    use crate::registers::register_tests::*;
    use crate::registers::*;
    use crate::user_ram::{RamAddress, user_ram_tests::*};
    use rstest::rstest;

    // 命令の種類
    #[derive(Clone, Copy, Debug)]
    pub enum ExampleInstruction {
        Add { id_rd: usize, id_rr: usize },
        Jmp { val_k: usize },
        Push { id_rr: usize },
        Pop { id_rd: usize },
        Nop,
    }

    impl ExampleInstruction {
        // ADD
        fn add<R: Registers>(registers: &mut R, id_rd: usize, id_rr: usize) -> RegisterUpdate {
            // 加算
            let rr = registers.read_from(RegisterType::General { id: id_rr });
            let rd = registers.read_from(RegisterType::General { id: id_rd });
            registers.add_to(RegisterType::General { id: id_rd }, rr);

            // status計算
            let r = registers.read_from(RegisterType::General { id: id_rd });

            let h = {
                let rd3 = rd.get_bit(3);
                let rr3 = rr.get_bit(3);
                let r3 = r.get_bit(3);

                rd3 && rr3 || rr3 && !r3 || !r3 && rd3
            };

            let v = {
                let rd7 = rd.get_bit(7);
                let rr7 = rr.get_bit(7);
                let r7 = r.get_bit(7);

                rd7 && rr7 && !r7 || !rd7 && !rr7 || r7
            };

            let n = r.get_bit(7);

            let s = n ^ v;

            let z = {
                !r.get_bit(7)
                    && !r.get_bit(6)
                    && !r.get_bit(5)
                    && !r.get_bit(4)
                    && !r.get_bit(3)
                    && !r.get_bit(2)
                    && !r.get_bit(1)
                    && !r.get_bit(0)
            };

            let c = {
                let rd7 = rd.get_bit(7);
                let rr7 = rr.get_bit(7);
                let r7 = r.get_bit(7);

                rd7 && rr7 || rr7 && !r7 || !r7 && rd7
            };

            let flags = [
                None,
                None,
                Some(h),
                Some(s),
                Some(v),
                Some(n),
                Some(z),
                Some(c),
            ];

            // status更新
            registers.write_to(
                RegisterType::Status,
                registers
                    .read_from(RegisterType::Status)
                    .generate_from_bit(&flags),
            );

            RegisterUpdate::new(1, PointerUpdate::Increment)
        }

        // JMP
        fn jmp(val_k: usize) -> RegisterUpdate {
            // レジスタ更新を返す
            RegisterUpdate::new(3, PointerUpdate::Absolute(val_k))
        }

        // PUSH
        fn push<R: Registers, U: UserRam>(
            registers: &mut R,
            user_ram: &mut U,
            id_rr: usize,
        ) -> RegisterUpdate {
            // 値を取得
            let rr = registers.read_from(RegisterType::General { id: id_rr });

            // push
            user_ram.write_to(RamAddress(registers.read_sp()), rr);
            // stack pointer更新
            registers.update_sp(PointerUpdate::Decrement);

            // update
            RegisterUpdate::new(2, PointerUpdate::Increment)
        }

        fn pop<R: Registers, U: UserRam>(
            registers: &mut R,
            user_ram: &mut U,
            id_rd: usize,
        ) -> RegisterUpdate {
            // pop
            // スタックポインタ更新
            registers.update_sp(PointerUpdate::Increment);
            // 値取得
            let value = user_ram.read_from(RamAddress(registers.read_sp()));
            // レジスタに代入
            registers.write_to(RegisterType::General { id: id_rd }, value);

            RegisterUpdate::new(2, PointerUpdate::Increment)
        }
    }

    impl Instruction for ExampleInstruction {
        fn run<R: Registers, U: UserRam>(
            &self,
            registers: &mut R,
            user_ram: &mut U,
        ) -> RegisterUpdate {
            use ExampleInstruction::*;

            // 命令の実行
            match self {
                Add { id_rd, id_rr } => Self::add(registers, *id_rd, *id_rr),
                Jmp { val_k } => Self::jmp(*val_k),
                Push { id_rr } => Self::push(registers, user_ram, *id_rr),
                Pop { id_rd } => Self::pop(registers, user_ram, *id_rd),
                Nop => RegisterUpdate::new(1, PointerUpdate::Increment),
            }
        }
    }

    // 命令の実行テスト
    #[cfg(test)]
    mod run {
        use crate::user_ram::RamAddress;

        use super::*;

        // addの実行
        #[rstest]
        #[case::default([12,100], [5,31], 131, 0b00101100)]
        #[case::truncate([3,202], [25,123], 69, 0b00100001)]
        fn add(
            #[case] rd: [usize; 2],
            #[case] rr: [usize; 2],
            #[case] expected: usize,
            #[case] status: usize,
        ) {
            //  初期化
            let mut registers = ExampleRegisters::new();
            let mut user_ram = ExampleUserRam::new();
            registers
                .write_to(RegisterType::General { id: rd[0] }, rd[1])
                .write_to(RegisterType::General { id: rr[0] }, rr[1]);

            // 命令実行
            let instruction = ExampleInstruction::Add {
                id_rd: rd[0],
                id_rr: rr[0],
            };
            let result = instruction.run(&mut registers, &mut user_ram);

            // テスト
            // 実行結果
            assert_eq!(
                registers.read_from(RegisterType::General { id: rd[0] }),
                expected,
                "Rd is wrong"
            );
            // ステータス更新確認
            assert_eq!(
                registers.read_from(RegisterType::Status),
                status,
                "status is wrong"
            );
            // 実行結果
            assert_eq!(
                result,
                RegisterUpdate::new(1, PointerUpdate::Increment),
                "register update is wrong"
            );
        }

        // jmpの実行
        #[rstest]
        #[case::defalut(1001, 0b0000_0000)]
        fn jmp(#[case] k: usize, #[case] status: usize) {
            // 初期化
            let mut registers = ExampleRegisters::new();
            let mut user_ram = ExampleUserRam::new();
            registers.write_to(RegisterType::ProgramCounter, 100);

            // 命令実行
            let instruction = ExampleInstruction::Jmp { val_k: k };
            let result = instruction.run(&mut registers, &mut user_ram);

            assert_eq!(
                result,
                RegisterUpdate::new(3, PointerUpdate::Absolute(k)),
                "update is wrong"
            );
            assert_eq!(
                registers.read_from(RegisterType::Status),
                status,
                "status is wrong"
            );
        }

        // pushのテスト
        #[test]
        fn push() {
            // 初期化
            let mut registers = ExampleRegisters::new();
            let mut user_ram = ExampleUserRam::new();
            registers
                .update_sp(PointerUpdate::Absolute(0x7F3))
                .write_to(RegisterType::General { id: 7 }, 127);

            // 実行
            let instruction = ExampleInstruction::Push { id_rr: 7 };
            let result = instruction.run(&mut registers, &mut user_ram);

            // テスト
            assert_eq!(
                result,
                RegisterUpdate::new(2, PointerUpdate::Increment),
                "update is wrong"
            );
            assert_eq!(user_ram.read_from(RamAddress(0x7F3)), 127);
            assert_eq!(registers.read_sp(), 0x7F2);
            assert_eq!(registers.read_from(RegisterType::Status), 0b0000_0000);
        }

        // pop
        #[test]
        fn pop() {
            // 初期化
            let mut registers = ExampleRegisters::new();
            let mut user_ram = ExampleUserRam::new();
            registers.update_sp(PointerUpdate::Absolute(0x5F6));
            user_ram.write_to(RamAddress(0x5F7), 57);

            // pop実行
            let instruction = ExampleInstruction::Pop { id_rd: 12 };
            let result = instruction.run(&mut registers, &mut user_ram);

            // テスト
            assert_eq!(result, RegisterUpdate::new(2, PointerUpdate::Increment));
            assert_eq!(registers.read_from(RegisterType::General { id: 12 }), 57);
            assert_eq!(registers.read_sp(), 0x5F7);
            assert_eq!(registers.read_from(RegisterType::Status), 0b0000_0000);
        }

        #[test]
        fn nop() {
            // 初期化
            let mut registers = ExampleRegisters::new();
            let mut user_ram = ExampleUserRam::new();

            // 実行
            let instruction = ExampleInstruction::Nop;
            let result = instruction.run(&mut registers, &mut user_ram);

            // テスト
            assert_eq!(result, RegisterUpdate::new(1, PointerUpdate::Increment));
            assert_eq!(registers, ExampleRegisters::new());
            assert_eq!(user_ram, ExampleUserRam::new());
        }
    }
}
