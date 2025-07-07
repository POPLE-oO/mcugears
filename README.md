# mcugears

Microcontroller unit emulator written in Rust

## プロジェクト概要

- [ ] マイコンの**インタプリタ**エミュレータのコア部分を実装する
  - 配列外参照はpanicさせて速度重視
- [ ] Arduinoのインタプリタエミュレータを実装する
- [ ] 実際にArduino IDEでコンパイルしたバイナリ(`.hex`)を実行する
- [ ] バックグランドの同時実行を実装する
- [ ] tauri裏で動作させてロボットエミュレーションを作る

## TODO

- [ ] レジスタの実装
  - [x] レジスタ構造体の実装
    - 汎用レジスタ
    - ステータスレジスタ
    - プログラムカウンター
    - スタックポインター
    - IOレジスタ(→ 上記に含まれないものはIOレジスタ)
  - [x] レジスタ指定の列挙型実装
  - [ ] 操作関数の実装
    - [x] read
    - [x] write
    - [x] add
    - [x] sub
    - [x] div
    - [x] mul
    - [ ] bit演算
  - [ ] レジスタ更新関数の実装
    - [x] update_pc     // プログラムカウンター
    - [ ] update_timer  // タイマー
    - [x] update        // 更新の実行

- [x] RAMの実装
  - [x] RAM構造体の実装
  - [x] 操作関数の実装
    - [x] read
    - [x] write

- [ ] 命令の実装
  - [ ] 命令関数の実装
    - [x] ADD
    - [x] JMP
    - [ ] PUSH
    - [ ] POP
    - [ ] NOP
    - [ ] MOV
    - [ ] CMP
    - [ ] LD
    - [ ] ST
  - [ ] 副作用持つかの判定関数

- [ ] マイコン構造体の実装
  - [ ] マイコン構造体の実装
  - [ ] 操作関数の実装(→イテレータ?)

## 免責事項

このライブラリは、ATmega328Pマイクロコントローラのエミュレーションを目的として開発されたオープンソースプロジェクトです。

本ライブラリは、Microchip Technology Inc.（旧Atmel社）によって開発、承認、または提携しているものではありません。
「ATmega328P」はMicrochip Technology Inc.の商標です。

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
