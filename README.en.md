[日本語](README.md) | **English**

# Tint*

Tint* is a small visual development environment for observing the code generation process of [Cerune](https://github.com/Hokutaka/Cerune).

Write Cerune. Generate it. Look at it.

From a single screen, you can observe how the same source code is transformed into Cerune IR, different backends, Assembly, Object files, Bytecode, and VM execution.

![Tint*](images/image.png)

```text
Cerune source
     │
     ▼
   Tint*
     │
     ├─ Sources
     ├─ Cerune IR
     │
     ├─ C
     ├─ C ASM
     │
     ├─ LLVM IR
     ├─ LLVM ASM
     │
     ├─ WAT
     │
     ├─ QBE IR
     ├─ QBE ASM
     │
     ├─ Direct ASM
     ├─ Object
     │    ├─ Sections
     │    ├─ Symbols
     │    ├─ Origin Symbols
     │    └─ Relocations
     │
     ├─ Bytecode
     └─ VM Output
```

## Features

- Cerune syntax highlighting
- Open / save `.ceru` files
- Save As
- Rename the current source file
- Display loaded Sources
- Display Cerune IR
- Generate C
- Generate LLVM IR
- Generate WebAssembly Text (`.wat`)
- Generate QBE IR
- Generate x86-64 Assembly directly
- Generate Cerune Bytecode
- Run Bytecode with the Cerune VM
- Observe C ASM through Clang
- Observe LLVM ASM through Clang
- Observe QBE ASM through QBE
- Switch between Windows x64 / Linux x86-64 targets
- Display Origins / Provenance annotations
- Generate COFF / ELF Object files
- Inspect Object Sections / Symbols / Relocations
- Inspect Origin Symbols stored in Object files, grouped by NodeId
- Switch between generated representations and execution results using tabs
- Validate with Cerune before code generation
- Keyboard shortcuts

## Shortcuts

| Shortcut | Action |
| --- | --- |
| `Ctrl+O` | Open |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Enter` | Emit |

## Targets

Tint* can switch the target used for native code generation.

```text
Windows x64
  x86_64-pc-windows-msvc

Linux x86-64
  x86_64-unknown-linux-gnu
```

The selected target is used for outputs such as LLVM IR, Assembly generated through Clang, Direct ASM, and Object generation.

The QBE path currently uses Linux x86-64.

```text
Cerune
  └─ QBE IR
       └─ QBE amd64_sysv
            └─ QBE ASM
```

## Origins

When `Origins` is enabled, Cerune preserves Provenance information in generated artifacts.

LLVM IR and Direct ASM receive annotations that allow Cerune IR NodeIds and source locations to be traced through generated code.

```text
Cerune source
     │
     ▼
Cerune IR
  Node #N
     │
     ▼
LLVM IR / Direct ASM
     │
     ▼
Object Symbols
```

When Origins is enabled during Object generation, `cerune_origin_...` symbols are also preserved in the Object symbol table.

Tint* separates these from normal Symbols and displays them as `Origin Symbols` grouped by NodeId.

```text
=== Origin Symbols ===

Node #10
  0x00000057  cerune_origin_n10_main_7
  0x0000007d  cerune_origin_n10_main_15

Node #11
  0x0000008a  cerune_origin_n11_main_17
```

This makes it possible to trace Provenance from source code through Cerune IR, Assembly, and Object files.

## Object

Tint* uses Cerune's `emit-obj` to generate native Object files.

```text
Windows x64
  → COFF .obj

Linux x86-64
  → ELF .o
```

Generated Object files are not shown as raw binary dumps. Tint* parses and displays their structure.

```text
Target
Format
Architecture
Kind
Size

Sections
Symbols
Origin Symbols
Relocations
```

The generated files are relocatable objects and can be linked into executables by an external linker.

## Requirements

Tint* uses the Cerune CLI for validation, code generation, and VM execution.

First, install Cerune.

```sh
git clone https://github.com/Hokutaka/Cerune.git
cd Cerune
cargo install --path .
```

Make sure the `cerune` command is available in `PATH`.

```sh
cerune --version
```

If you are developing Cerune and Tint* at the same time, reinstall the CLI after making changes to Cerune.

```sh
cargo install --path . --force
```

Developing Tint* also requires the usual Tauri development environment for your platform.

### Clang

`C ASM` and `LLVM ASM` are generated using Clang.

Make sure `clang` is available in `PATH`.

```sh
clang --version
```

Tint* also passes the selected Cerune target to Clang.

```text
x86_64-pc-windows-msvc
x86_64-unknown-linux-gnu
```

### QBE

QBE is used to generate `QBE ASM`.

In the current Windows environment, Tint* invokes QBE through WSL.

```sh
wsl qbe
```

QBE IR itself is generated directly by Cerune.

QBE is only required when that IR is further converted into Assembly for display as `QBE ASM`.

The QBE path uses Linux x86-64 / `amd64_sysv`.

## Development

Clone the Tint* repository and install the frontend dependencies.

```sh
npm install
```

Start the app in development mode.

```sh
npm run tauri dev
```

Build the application.

```sh
npm run tauri build
```

To check only the Rust side, run the following from `src-tauri`.

```sh
cd src-tauri
cargo fmt
cargo check
```

## How it works

Tint* does not embed Cerune's compiler implementation inside the application.

When you Emit, Tint* prepares a temporary `.ceru` source file and invokes the installed Cerune CLI.

```text
Tint*
 │
 ├─ cerune check
 ├─ cerune emit-sources
 ├─ cerune emit-ir
 ├─ cerune emit-c
 ├─ cerune emit-llvm
 ├─ cerune emit-wat
 ├─ cerune emit-qbe
 ├─ cerune emit-asm
 ├─ cerune emit-obj
 ├─ cerune emit-bytecode
 └─ cerune run
```

Some views are generated directly by Cerune.

```text
Cerune → Sources
Cerune → Cerune IR
Cerune → C
Cerune → LLVM IR
Cerune → WAT
Cerune → QBE IR
Cerune → Direct ASM
Cerune → Object
Cerune → Bytecode
Cerune → Bytecode → Cerune VM → VM Output
```

Other views intentionally show transformations performed by external tools.

```text
Cerune → C
       → Clang
       → C ASM

Cerune → LLVM IR
       → Clang
       → LLVM ASM

Cerune → QBE IR
       → QBE
       → QBE ASM
```

Tint* parses the COFF / ELF Object generated by Cerune.

```text
Cerune
  ↓
Object
  ↓
Tint*
  ├─ Sections
  ├─ Symbols
  ├─ Origin Symbols
  └─ Relocations
```

This makes it possible to observe:

```text
the same Cerune source
        ↓
different lowering / backend / execution paths
        ↓
the same Provenance across them
```

from a single screen.

## Design

Tint* is intentionally small.

It is not intended to be a general-purpose, full-featured IDE.

It is a lightweight visual environment for writing Cerune and observing how the same source code changes across different representations and execution paths.

The interface fundamentally focuses on:

```text
source | representation / output
```

Cerune is responsible for the language, semantic analysis, IR, code generation, Bytecode, VM, and Provenance.

External tools such as Clang and QBE are responsible for their respective additional transformations.

Tint* is a window for observing all of them together.

## Name

`tint` means a pale color made by adding white to a color.
