# mcugears

Microcontroller emulator written in Rust.

## TODO

- スタック、ヒープの実装
- push,pop命令,alloc関係(SRAM)の実装
- 未実装の命令を実装する(特に副作用のあるもの)
- Mcuのtest実装

## プロジェクト構成

```dir
mcugears/               // workspace
├── Cargo.toml
├── LICENSE
├── README.md           // ← 現在の場所
├── crates
│   ├── mcugears_core/  // 基本機能
│   └── mcugears_atmega328p/   // coreをatmega328p向けに実装する
├── target/
└── tests/              // 組み合わせテスト
```

## 概要

- [ ] Arduinoのエミュレータを作ってみる
- [ ] Arduino IDEでコンパイルしたバイナリ(`.hex`)を実行する
- [ ] バックグラウンドで複数台同時に動かしたい
- [ ] それを通信させたい
- [ ] 完成したら他の固定長命令のエミュレータを実装したい
- [ ] それをtauriのバックグラウンドで動作させてロボットエミュレータを作りたい

## 構成

### core

基本機能

- マイコンエミュ本体
マイコンの現在のステータスを保持する構造体

```rust
struct Mcu{
    commands: Vec<Command>      // 命令のベクトル
    registers: Registers,       // レジスタの構造体
    //...
}
```

- 命令
単一の命令のEnum

```rust
trait Command{/*略*/}
```

- レジスタの構造体
レジスタの状態を持つ構造体

```rust
trait Registers{/*略*/}
```

### アーキテクチャごとの固有実装

アーキテクチャごとの仕様を実装する

- バイナリパーサー
.hexを`Command`にパースして`CommandsIterator`を構成する

- 命令(enum)
命令の種類のEnumに`Command`を実装する

```rust
enum ExampleCommand{
    Add(u8,u8),     // レジスタ情報など実行に必要な情報を持つ
    //...
}

impl Command for ExampleCommand{}
```

- 命令の実装
`Command`に`match`で実装する

- レジスタ(struct)
レジスタ構成の構造体に`Registers`を実装する

```rust
struct ExampleRegisters{
    r: Vec<u8>,
    //...
}

impl Registers for ExampleRegisters{}
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
