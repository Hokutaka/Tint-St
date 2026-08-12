[英語](README.md) | *日本語*

# Tint*

**Tint** は、[Primer](https://github.com/Hokutaka/Primer) をちょっと気軽に書いて、生成されたコードを眺めるための小さな開発環境です。

Primerのソースを書くと、同じコードから生成された **C / LLVM IR / WAT** を横で見ることができます。

```text
Primer
  │
  ▼
 Tint*
  │
  ├─ C
  ├─ LLVM IR
  └─ WAT
```

大きなIDEを目指しているわけではありません。

**書く。生成する。眺める。**

ための、小さくてシンプルな道具です。

## できること

* `.prim` ファイルを開く
* Primerを書く
* シンタックスハイライト
* ファイルを保存 / 名前を付けて保存
* 開いているファイルの名前を変更
* Primerのコードをチェック
* Cコードを生成
* LLVM IRを生成
* WebAssembly Text (`.wat`) を生成
* C / LLVM IR / WAT をタブで切り替えて眺める

## ショートカット

| キー                 | 動作       |
| ------------------ | -------- |
| `Ctrl + O`         | ファイルを開く  |
| `Ctrl + S`         | 保存       |
| `Ctrl + Shift + S` | 名前を付けて保存 |
| `Ctrl + Enter`     | Emit     |

## Primerについて

Tint自身がPrimerをコンパイルしているわけではありません。

実際の言語処理は、インストールされている **Primer CLI** に任せています。

```text
Tint
 │
 ├─ primer check
 ├─ primer emit-c
 ├─ primer emit-llvm
 └─ primer emit-wat
```

Tintはその結果を受け取って、画面に表示します。

そのため、先にPrimerをインストールしておく必要があります。

```sh
git clone https://github.com/Hokutaka/Primer.git
cd Primer
cargo install --path .
```

確認：

```sh
primer --version
```

これが動けば準備OKです。

## Tintを動かす

リポジトリを取得します。

```sh
git clone https://github.com/Hokutaka/Tint.git
cd Tint
```

依存関係をインストール：

```sh
npm install
```

開発モードで起動：

```sh
npm run tauri dev
```

ビルド：

```sh
npm run tauri build
```

## Tintという名前

**Tint** には、白に少し色を加えた淡い色、という意味があります。
小さくシンプルな画面に、ほんの少し色を付ける。
そんなイメージからこの名前になりました。

## 方針

Tintは、できるだけ小さいままにします。
Primerを書く場所と、Primerが生成したものを見る場所。
基本はそれだけです。

```text
source | generated code
```

![alt text](/images/image.png)

Primerが言語そのものを担当して、Tintはそれを覗くためのものになります。
