**English** | [日本語](README.ja.md)

# Tint*

Tint is a small visual development environment for [Primer](https://github.com/Hokutaka/Primer).

It provides a simple place to edit Primer source code and observe the representations generated from it.

```text
Primer source
     │
     ▼
   Tint*
     │
     ├─ C
     ├─ LLVM IR
     └─ WAT
```

## Features

* Primer syntax highlighting
* Open and save `.prim` files
* Save As
* Rename the current source file
* Generate C
* Generate LLVM IR
* Generate WebAssembly Text (`.wat`)
* Switch between generated representations
* Primer validation before code generation
* Keyboard shortcuts

## Shortcuts

| Shortcut       | Action  |
| -------------- | ------- |
| `Ctrl+O`       | Open    |
| `Ctrl+S`       | Save    |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Enter`   | Emit    |

## Requirements

Tint uses the Primer CLI for validation and code generation.

Install Primer first:

```sh
git clone https://github.com/Hokutaka/Primer.git
cd Primer
cargo install --path .
```

Make sure the `primer` command is available in your `PATH`:

```sh
primer --version
```

Tint also requires the usual Tauri development dependencies for your platform.

## Development

Clone Tint and install the frontend dependencies:

```sh
git clone https://github.com/Hokutaka/Tint.git
cd Tint
npm install
```

Run Tint in development mode:

```sh
npm run tauri dev
```

Build the application:

```sh
npm run tauri build
```

## How it works

Tint intentionally keeps Primer itself outside the application.

When code is emitted, Tint creates a temporary `.prim` source file and calls the installed Primer CLI:

```text
Tint
 │
 ├─ primer check
 ├─ primer emit-c
 ├─ primer emit-llvm
 └─ primer emit-wat
```

The generated representations are captured from standard output and displayed directly in Tint.

Tint does not need to keep generated `.c`, `.ll`, or `.wat` files on disk.

## Design

Tint is intentionally small.

It is not intended to become a full general-purpose IDE. Its purpose is to provide a lightweight visual environment for writing Primer and observing how the same source is lowered into different representations.

The interface therefore stays focused on two things:

```text
source | generated representation
```

![alt text](/images/image.png)

Primer remains responsible for the language and code generation.

Tint remains the window into it.
