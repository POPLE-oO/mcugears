# mcugears

Microcontroller emulator written in Rust.

## プロジェクト構成

```dir
mcugears/               // workspace
├── Cargo.lock  
├── Cargo.toml
├── LICENSE
├── README.md           // ← 現在の場所
├── crates
│   ├── mcugears_core/  // 基本機能
│   └── mcugears_avr/   // coreを実装しavrアーキテクチャに対応する
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
    commands: CommandsIterator, // 命令のイテレータ
    registers: Registers,       // レジスタの構造体
    //...
}
```

- 命令のイテレータ
バイナリをパースした命令のイテレータ

```rust
trait CommandsIterator{/*略*/}
```

- 命令
`CommandsIterator`を構成する単一の命令のEnum

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
