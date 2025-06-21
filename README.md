# mcugears

Microcontroller emulator written in Rust.

## 概要

- [ ] Arduinoのエミュレータを作ってみる
- [ ] Arduino IDEでコンパイルしたバイナリ(`.hex`)を実行する
- [ ] バックグラウンドで複数台同時に動かしたい
- [ ] それを通信させたい
- [ ] 完成したら他の固定長命令のエミュレータを実装したい
- [ ] それをtauriのバックグラウンドで動作させてロボットエミュレータを作りたい

## TODO

- [x] レジスタの実装,テスト実装
- [x] 命令の実装,テスト実装
- [x] スタック、ヒープの実装, テスト実装
- [ ] push(o),pop命令(o),alloc関係(x)(SRAM)の実装
- [ ] 未実装の命令を実装する(特に副作用(IO)のあるもの)
- [ ] 命令の実装
  - [x] ADD
  - [x] JMP
  - [x] PUSH
  - [x] POP
  - [x] NOP
  - [x] EMPTY(2x16bitの命令を[命令, EMPTY]と表す用)
  - [x] EMPTY(2x16bitの命令を[命令, EMPTY]と表す用)
  - [ ] MV
  - [ ] CMP
- [ ] マイコン構造体の実装
- [ ] Mcuのtest実装
- [ ] CMPなどほかに必要そうな基本的な命令を作成
- [ ] 簡単なアセンブリっぽくプログラムを作ってエミュレートしてみる
- [ ] mcugears_328pの実装

## 免責事項 (Disclaimer)

このライブラリは、ATmega328Pマイクロコントローラのエミュレーションを目的として開発されたオープンソースプロジェクトです。

本ライブラリは、Microchip Technology Inc.（旧Atmel社）によって開発、承認、または提携しているものではありません。
「ATmega328P」はMicrochip Technology Inc.の商標です。

## プロジェクト構成

```dir
mcugears/               // workspace
├── Cargo.toml
├── LICENSE
├── README.md           // ← 現在の場所
├── crates
│   ├── mcugears_core/  // 基本機能
│   └── mcugears_328p/   // coreをatmega328p向けに実装する
├── target/
└── tests/              // 組み合わせテスト
```

## AVR

とりあえずAtmega328pのアーキテクチャを実装したい

### register構成

とりあえずレジスタ構成は以下に従えばよさそう
[ATmega328P DATASHEET](https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf)

- 「30.Register Summary」にたぶん全体が載ってる
- Reserved はエミュ上では気にしなくていいと思う(命令さえ実現すればいいから)
- 詳細に関しては表の右側にページで乗ってる

### instruction

opcodeとかはデータシートじゃなくてアーキテクチャの説明に書いてあるっぽい
ATmega328pはAVRe+アーキテクチャらしいのでAVRアーキテクチャの仕様書から必要なものを実装する

[AVR Instruction Set Manual](https://ww1.microchip.com/downloads/aemDocuments/documents/MCU08/ProductDocuments/ReferenceManuals/AVR-InstructionSet-Manual-DS40002198.pdf)

- 具体的なbit列やstatusレジスタまで乗ってたのでこれでいいと思う
- ボードごとに微妙に仕様が違って最後のほうにそれが載ってるから注意

### stack,heap

- とりあえず`PUSH`,`POP`さえ用意すれば動きそう
- 正直まだよくわかってないけどSRAMの操作にかかわる命令を実装すればよさそう?
