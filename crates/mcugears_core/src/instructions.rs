use crate::{
    registers::{RegisterUpdate, Registers},
    user_ram::UserRam,
};

// 命令のメソッド
pub trait Instruction {
    fn run<R: Registers, D: UserRam>(&self, registers: &mut R, user_ram: &mut D) -> RegisterUpdate;
}

#[cfg(test)]
pub mod instructions_tests {
    use super::*;
    use crate::registers::register_tests::*;
    use crate::registers::*;
    use crate::user_ram::user_ram_tests::*;
    use rstest::rstest;

    // 命令の種類
    #[derive(Clone, Copy, Debug)]
    pub enum ExampleInstruction {
        Add { id_rd: usize, id_rr: usize },
        Jmp { val_k: usize },
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

            RegisterUpdate::new(1, PCUpdate::Default)
        }

        // JMP
        fn jmp(val_k: usize) -> RegisterUpdate {
            RegisterUpdate::new(3, PCUpdate::Absolute(val_k))
        }
    }

    impl Instruction for ExampleInstruction {
        fn run<R: Registers, D: UserRam>(
            &self,
            registers: &mut R,
            user_ram: &mut D,
        ) -> RegisterUpdate {
            // 命令の実行
            use ExampleInstruction::*;

            match self {
                Add { id_rd, id_rr } => Self::add(registers, *id_rd, *id_rr),
                Jmp { val_k } => Self::jmp(*val_k),
            }
        }
    }

    // 命令の実行テスト
    #[cfg(test)]
    mod run {
        use crate::instructions;

        use super::*;

        // addの実行
        #[rstest]
        #[case::default([12,100], [5,31], 131, 0b00101100, 1,PCUpdate::Default)]
        fn add(
            #[case] rd: [usize; 2],
            #[case] rr: [usize; 2],
            #[case] expected: usize,
            #[case] status: usize,
            #[case] cycles: usize,
            #[case] pc_update: PCUpdate,
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
                RegisterUpdate::new(cycles, pc_update),
                "register update is wrong"
            );
        }

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

            assert_eq!(result, RegisterUpdate::new(3, PCUpdate::Absolute(k)));
        }
    }
}
