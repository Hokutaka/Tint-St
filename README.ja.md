[English](README.md) | **日本語**

# Tint\*

Tint は [Primer](https://github.com/Hokutaka/Primer) のための、小さなビジュアル開発環境です。

Primer のコードを書く。生成する。眺める。

同じソースコードが、異なる表現や実行経路へどう変換されていくのかを、ひとつの画面から観察できます。

![Tint](images/image.png)

```text
Primer source
     │
     ▼
   Tint*
     │
     ├─ C
     ├─ C ASM
     ├─ LLVM IR
     ├─ LLVM ASM
     ├─ WAT
     ├─ QBE IR
     ├─ QBE ASM
     ├─ Direct ASM
     ├─ Bytecode
     └─ VM Output
```

## 機能

- Primer のシンタックスハイライト
- `.prim` ファイルを開く / 保存する
- 名前を付けて保存
- 現在のソースファイルをリネーム
- C を生成
- LLVM IR を生成
- WebAssembly Text (`.wat`) を生成
- QBE IR を生成
- x86-64 Assembly を直接生成
- Primer Bytecode を生成
- Primer VM で Bytecode を実行
- Clang を経由して C ASM を観察
- Clang を経由して LLVM ASM を観察
- QBE を経由して QBE ASM を観察
- 生成された表現や実行結果をタブで切り替え
- コード生成前に Primer で検証
- キーボードショートカット

## ショートカット

| ショートカット | 操作 |
| -------------- | ---- |
| `Ctrl+O` | 開く |
| `Ctrl+S` | 保存 |
| `Ctrl+Shift+S` | 名前を付けて保存 |
| `Ctrl+Enter` | Emit |

## 必要なもの

Tint は、コードの検証・生成・VM 実行に Primer CLI を使用します。

まず Primer をインストールします。

```sh
git clone https://github.com/Hokutaka/Primer.git
cd Primer
cargo install --path .
```

`primer` コマンドが `PATH` から利用できることを確認します。

```sh
primer --version
```

Primer と Tint を同時に開発している場合、Primer 側を変更したあとは CLI を再インストールします。

```sh
cargo install --path . --force
```

Tint の開発には、このほか各プラットフォームで通常必要となる Tauri の開発環境が必要です。

### Clang

`C ASM` と `LLVM ASM` は Clang を使って生成します。

`clang` が `PATH` から利用できることを確認してください。

```sh
clang --version
```

### QBE

`QBE ASM` の生成には QBE を使用します。

現在の Windows 環境では、Tint は WSL 経由で QBE を呼び出します。

```sh
wsl qbe
```

QBE IR 自体は Primer が直接生成します。

QBE が必要なのは、その IR をさらに Assembly へ変換して `QBE ASM` として表示するときだけです。

QBE ASM の生成が利用できない場合でも、ほかの Primer の生成結果は表示できます。

## 開発

Tint を clone して、フロントエンドの依存関係をインストールします。

```sh
git clone https://github.com/Hokutaka/Tint.git
cd Tint
npm install
```

開発モードで起動します。

```sh
npm run tauri dev
```

アプリケーションをビルドします。

```sh
npm run tauri build
```

## 仕組み

Tint は、Primer のコンパイラ実装そのものをアプリケーション内部へ持ち込みません。

Emit すると、一時的な `.prim` ソースファイルを作成し、インストールされている Primer CLI を呼び出します。

```text
Tint
 │
 ├─ primer check
 ├─ primer emit-c
 ├─ primer emit-llvm
 ├─ primer emit-wat
 ├─ primer emit-qbe
 ├─ primer emit-asm
 ├─ primer emit-bytecode
 └─ primer run
```

いくつかの表示は Primer が直接生成します。

```text
Primer → C
Primer → LLVM IR
Primer → WAT
Primer → QBE IR
Primer → Direct ASM
Primer → Bytecode
Primer → Bytecode → Primer VM → VM Output
```

一方で、外部ツールによる変換結果を意図的に表示しているものもあります。

```text
Primer → C → Clang → C ASM
Primer → LLVM IR → Clang → LLVM ASM
Primer → QBE IR → QBE → QBE ASM
```

こうすることで、

```text
同じ Primer source
        ↓
異なる lowering / backend / execution path
```

をひとつの画面から観察できます。

生成された表現や実行結果は Tint が受け取り、そのまま画面に表示します。

外部変換のために使用する一時ファイルは、Primer のソースプロジェクトには残しません。

## 設計

Tint は意図的に小さく作られています。

汎用的なフル機能 IDE を目指すものではありません。

Primer を書き、同じソースコードが異なる表現や実行経路へどう変わっていくのかを眺めるための、軽量なビジュアル環境です。

インターフェースが注目するものも、基本的にはこの2つです。

```text
source | representation / output
```

Primer は、言語・コード生成・Bytecode・VM を担当します。

Clang や QBE などの外部ツールは、それぞれの変換を担当します。

Tint は、それらを覗くための窓です。

## 名前

`tint` は、色に白を加えて明るくした淡い色を意味します。
