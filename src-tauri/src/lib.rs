use object::{Object, ObjectSection, ObjectSymbol};
use serde::Serialize;
use std::fmt::Write as _;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EmitResult {
    sources: String,
    ir: String,

    c: String,
    c_asm: String,

    llvm: String,
    llvm_asm: String,

    wat: String,

    qbe: String,
    qbe_asm: String,

    direct_asm: String,
    object: String,

    bytecode: String,
    vm_output: String,
}

fn run_cerune(command: &str, source_path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("cerune")
        .arg(command)
        .arg(source_path)
        .args(args)
        .output()
        .map_err(|error| {
            format!(
                "failed to start Cerune: {error}\n\
                     Make sure `cerune` is installed and available in PATH."
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        let stdout = String::from_utf8_lossy(&output.stdout);

        let message = if !stderr.trim().is_empty() {
            stderr.trim().to_owned()
        } else if !stdout.trim().is_empty() {
            stdout.trim().to_owned()
        } else {
            format!("Cerune `{command}` failed without an error message")
        };

        return Err(message);
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("Cerune returned invalid UTF-8: {error}"))
}

fn run_cerune_object(
    source_path: &Path,
    target: &str,
    annotate_origins: bool,
    timestamp: u128,
) -> Result<String, String> {
    let extension = match target {
        "x86_64-pc-windows-msvc" => "obj",
        "x86_64-unknown-linux-gnu" => "o",
        _ => {
            return Err(format!("unsupported object target: {target}"));
        }
    };

    let object_path =
        std::env::temp_dir().join(format!("tint-{}-{timestamp}.{extension}", process::id(),));

    let result = (|| {
        let mut command = Command::new("cerune");

        command
            .arg("emit-obj")
            .arg(source_path)
            .arg("--target")
            .arg(target);

        if annotate_origins {
            command.arg("--annotate-origins");
        }

        let output = command
            .arg("-o")
            .arg(&object_path)
            .output()
            .map_err(|error| format!("failed to start Cerune: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            let stdout = String::from_utf8_lossy(&output.stdout);

            let message = if !stderr.trim().is_empty() {
                stderr.trim().to_owned()
            } else if !stdout.trim().is_empty() {
                stdout.trim().to_owned()
            } else {
                "Cerune `emit-obj` failed without an error message".to_owned()
            };

            return Err(message);
        }

        let bytes = fs::read(&object_path)
            .map_err(|error| format!("failed to read generated object: {error}"))?;

        describe_object(&bytes, target)
    })();

    let _ = fs::remove_file(&object_path);

    result
}

fn origin_node_id(name: &str) -> Option<u64> {
    let rest = name.strip_prefix("cerune_origin_n")?;

    let node = rest.split('_').next()?;

    node.parse().ok()
}

fn describe_object(bytes: &[u8], target: &str) -> Result<String, String> {
    let file =
        object::File::parse(bytes).map_err(|error| format!("failed to parse object: {error}"))?;

    let mut output = String::new();

    writeln!(output, "Target: {target}",).unwrap();

    writeln!(output, "Format: {:?}", file.format(),).unwrap();

    writeln!(output, "Architecture: {:?}", file.architecture(),).unwrap();

    writeln!(output, "Kind: {:?}", file.kind(),).unwrap();

    writeln!(output, "Little endian: {}", file.is_little_endian(),).unwrap();

    writeln!(output, "Size: {} bytes", bytes.len(),).unwrap();

    writeln!(output).unwrap();

    writeln!(output, "=== Sections ===",).unwrap();

    for section in file.sections() {
        let name = section.name().unwrap_or("<non-utf8>");

        writeln!(
            output,
            "{:?}  {:<20} \
             kind={:?} size={} align={}",
            section.index(),
            name,
            section.kind(),
            section.size(),
            section.align(),
        )
        .unwrap();
    }

    writeln!(output).unwrap();

    writeln!(output, "=== Symbols ===",).unwrap();

    let mut origins: BTreeMap<u64, Vec<(u64, String)>> = BTreeMap::new();

    for symbol in file.symbols() {
        let name = symbol.name().unwrap_or("<non-utf8>");

        if let Some(node_id) = origin_node_id(name) {
            origins
                .entry(node_id)
                .or_default()
                .push((symbol.address(), name.to_owned()));

            continue;
        }

        writeln!(
            output,
            "0x{:08x}  {:<32} \
            kind={:?} scope={:?} size={} section={:?}",
            symbol.address(),
            name,
            symbol.kind(),
            symbol.scope(),
            symbol.size(),
            symbol.section_index(),
        )
        .unwrap();
    }

    writeln!(output).unwrap();

    if !origins.is_empty() {
        writeln!(output).unwrap();

        writeln!(output, "=== Origin Symbols ===",).unwrap();

        for (node_id, symbols) in origins {
            writeln!(output, "Node #{node_id}",).unwrap();

            for (address, name) in symbols {
                writeln!(output, "  0x{address:08x}  {name}",).unwrap();
            }
        }
    }

    writeln!(output, "=== Relocations ===",).unwrap();

    for section in file.sections() {
        let name = section.name().unwrap_or("<non-utf8>");

        for (offset, relocation) in section.relocations() {
            writeln!(
                output,
                "{name}+0x{offset:04x}  \
                 kind={:?} encoding={:?} \
                 size={} target={:?} addend={}",
                relocation.kind(),
                relocation.encoding(),
                relocation.size(),
                relocation.target(),
                relocation.addend(),
            )
            .unwrap();
        }
    }

    Ok(output)
}

fn run_qbe_asm(qbe: &str, timestamp: u128) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();

    let stem = format!("tint-{}-{timestamp}", process::id());

    let qbe_path = temp_dir.join(format!("{stem}.ssa"));

    let asm_path = temp_dir.join(format!("{stem}.qbe.s"));

    fs::write(&qbe_path, qbe)
        .map_err(|error| format!("failed to create temporary QBE IR: {error}"))?;

    let result = (|| {
        let qbe_name = qbe_path
            .file_name()
            .ok_or_else(|| "invalid QBE temporary path".to_owned())?;

        let asm_name = asm_path
            .file_name()
            .ok_or_else(|| "invalid ASM temporary path".to_owned())?;

        let output = Command::new("wsl")
            .current_dir(&temp_dir)
            .arg("qbe")
            .arg("-t")
            .arg("amd64_sysv")
            .arg("-o")
            .arg(asm_name)
            .arg(qbe_name)
            .output()
            .map_err(|error| format!("failed to start QBE through WSL: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            let stdout = String::from_utf8_lossy(&output.stdout);

            let message = if !stderr.trim().is_empty() {
                stderr.trim().to_owned()
            } else if !stdout.trim().is_empty() {
                stdout.trim().to_owned()
            } else {
                "QBE failed without an error message".to_owned()
            };

            return Err(message);
        }

        fs::read_to_string(&asm_path)
            .map_err(|error| format!("failed to read QBE assembly: {error}"))
    })();

    let _ = fs::remove_file(&qbe_path);
    let _ = fs::remove_file(&asm_path);

    result
}

fn run_clang_asm(
    source: &str,
    extension: &str,
    kind: &str,
    timestamp: u128,
    target: &str,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();

    let stem = format!("tint-{}-{timestamp}", process::id());

    let input_path = temp_dir.join(format!("{stem}.{extension}"));

    let asm_path = temp_dir.join(format!("{stem}.{kind}.s"));

    fs::write(&input_path, source)
        .map_err(|error| format!("failed to create temporary {kind} input: {error}"))?;

    let result = (|| {
        let output = Command::new("clang")
            .arg("-S")
            .arg("-O0")
            .arg(format!("--target={target}"))
            .arg(&input_path)
            .arg("-o")
            .arg(&asm_path)
            .output()
            .map_err(|error| {
                format!(
                    "failed to start Clang: {error}\n\
                         Make sure `clang` is available in PATH."
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            let stdout = String::from_utf8_lossy(&output.stdout);

            let message = if !stderr.trim().is_empty() {
                stderr.trim().to_owned()
            } else if !stdout.trim().is_empty() {
                stdout.trim().to_owned()
            } else {
                "Clang failed without an error message".to_owned()
            };

            return Err(message);
        }

        fs::read_to_string(&asm_path)
            .map_err(|error| format!("failed to read generated assembly: {error}"))
    })();

    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&asm_path);

    result
}

#[tauri::command]
fn rename_source(path: String, new_name: String) -> Result<String, String> {
    let new_name = new_name.trim();

    if new_name.is_empty() {
        return Err("file name cannot be empty".to_owned());
    }

    if new_name.contains('/') || new_name.contains('\\') {
        return Err("file name cannot contain path separators".to_owned());
    }

    let new_name = if new_name.ends_with(".ceru") {
        new_name.to_owned()
    } else {
        format!("{new_name}.ceru")
    };

    let old_path = std::path::PathBuf::from(path);

    let parent = old_path
        .parent()
        .ok_or_else(|| "file has no parent directory".to_owned())?;

    let new_path = parent.join(new_name);

    if new_path.exists() {
        return Err("a file with that name already exists".to_owned());
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|error| format!("failed to rename file: {error}"))?;

    Ok(new_path.to_string_lossy().into_owned())
}

#[tauri::command]
fn emit_all(
    source: String,
    target: String,
    annotate_origins: bool,
    source_path: Option<String>,
) -> Result<EmitResult, String> {
    match target.as_str() {
        "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu" => {}

        _ => {
            return Err(format!("unsupported target: {target}"));
        }
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let filename = format!(".tint-{}-{timestamp}.ceru", process::id(),);

    let source_path = if let Some(path) = source_path {
        let path = std::path::PathBuf::from(path);

        let parent = path
            .parent()
            .ok_or_else(|| "source file has no parent directory".to_owned())?;

        parent.join(filename)
    } else {
        std::env::temp_dir().join(filename)
    };

    fs::write(&source_path, source)
        .map_err(|error| format!("failed to create temporary Cerune source: {error}"))?;

    let result = (|| {
        run_cerune("check", &source_path, &[])?;

        let sources = run_cerune("emit-sources", &source_path, &[])?;

        let ir = run_cerune("emit-ir", &source_path, &[])?;

        let c = run_cerune("emit-c", &source_path, &[])?;

        let c_asm = run_clang_asm(&c, "c", "c", timestamp, &target)
            .unwrap_or_else(|error| format!("C ASM unavailable:\n{error}"));

        let llvm = if annotate_origins {
            run_cerune(
                "emit-llvm",
                &source_path,
                &["--target", &target, "--annotate-origins"],
            )?
        } else {
            run_cerune("emit-llvm", &source_path, &["--target", &target])?
        };

        let llvm_asm = run_clang_asm(&llvm, "ll", "llvm", timestamp, &target)
            .unwrap_or_else(|error| format!("LLVM ASM unavailable:\n{error}"));

        let wat = run_cerune("emit-wat", &source_path, &[])?;

        let qbe = run_cerune(
            "emit-qbe",
            &source_path,
            &["--target", "x86_64-unknown-linux-gnu"],
        )?;

        let qbe_asm = run_qbe_asm(&qbe, timestamp)
            .unwrap_or_else(|error| format!("QBE ASM unavailable:\n{error}"));

        let direct_asm = if annotate_origins {
            run_cerune(
                "emit-asm",
                &source_path,
                &["--target", &target, "--annotate-origins"],
            )?
        } else {
            run_cerune("emit-asm", &source_path, &["--target", &target])?
        };

        let object = run_cerune_object(&source_path, &target, annotate_origins, timestamp)?;

        let bytecode = run_cerune("emit-bytecode", &source_path, &[])?;

        let vm_output = run_cerune("run", &source_path, &[])?;

        Ok(EmitResult {
            sources,
            ir,
            c,
            c_asm,
            llvm,
            llvm_asm,
            wat,
            qbe,
            qbe_asm,
            direct_asm,
            object,
            bytecode,
            vm_output,
        })
    })();

    let _ = fs::remove_file(&source_path);

    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![emit_all, rename_source,])
        .run(tauri::generate_context!())
        .expect("error while running Tint");
}
