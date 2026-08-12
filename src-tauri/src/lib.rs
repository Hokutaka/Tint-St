use serde::Serialize;
use std::{
    fs,
    path::Path,
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EmitResult {
    c: String,
    llvm: String,
    wat: String,
    qbe: String,
    qbe_asm: String,
}

fn run_primer(command: &str, source_path: &Path) -> Result<String, String> {
    let output = Command::new("primer")
        .arg(command)
        .arg(source_path)
        .output()
        .map_err(|error| {
            format!(
                "failed to start Primer: {error}\n\
                 Make sure `primer` is installed and available in PATH."
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        let stdout = String::from_utf8_lossy(&output.stdout);

        let message = if !stderr.trim().is_empty() {
            stderr.trim().to_owned()
        } else {
            stdout.trim().to_owned()
        };

        return Err(message);
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("Primer returned invalid UTF-8: {error}"))
}

fn run_qbe_asm(
    qbe: &str,
    timestamp: u128,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();

    let stem =
        format!("tint-{}-{timestamp}", process::id());

    let qbe_path =
        temp_dir.join(format!("{stem}.ssa"));

    let asm_path =
        temp_dir.join(format!("{stem}.qbe.s"));

    fs::write(&qbe_path, qbe)
        .map_err(|error| {
            format!(
                "failed to create temporary QBE IR: {error}"
            )
        })?;

    let result = (|| {
        let qbe_name =
            qbe_path
                .file_name()
                .ok_or_else(|| {
                    "invalid QBE temporary path"
                        .to_owned()
                })?;

        let asm_name =
            asm_path
                .file_name()
                .ok_or_else(|| {
                    "invalid ASM temporary path"
                        .to_owned()
                })?;

        let output =
            Command::new("wsl")
                .current_dir(&temp_dir)
                .arg("qbe")
                .arg("-t")
                .arg("amd64_win")
                .arg("-o")
                .arg(asm_name)
                .arg(qbe_name)
                .output()
                .map_err(|error| {
                    format!(
                        "failed to start QBE through WSL: {error}"
                    )
                })?;

        if !output.status.success() {
            let stderr =
                String::from_utf8_lossy(
                    &output.stderr,
                );

            let stdout =
                String::from_utf8_lossy(
                    &output.stdout,
                );

            let message =
                if !stderr.trim().is_empty() {
                    stderr.trim().to_owned()
                } else {
                    stdout.trim().to_owned()
                };

            return Err(
                if message.is_empty() {
                    "QBE failed without an error message"
                        .to_owned()
                } else {
                    message
                },
            );
        }

        fs::read_to_string(&asm_path)
            .map_err(|error| {
                format!(
                    "failed to read QBE assembly: {error}"
                )
            })
    })();

    let _ = fs::remove_file(&qbe_path);
    let _ = fs::remove_file(&asm_path);

    result
}

#[tauri::command]
fn rename_source(
    path: String,
    new_name: String,
) -> Result<String, String> {
    let new_name = new_name.trim();

    if new_name.is_empty() {
        return Err(
            "file name cannot be empty".to_owned()
        );
    }

    if new_name.contains('/')
        || new_name.contains('\\')
    {
        return Err(
            "file name cannot contain path separators"
                .to_owned()
        );
    }

    let new_name =
        if new_name.ends_with(".prim") {
            new_name.to_owned()
        } else {
            format!("{new_name}.prim")
        };

    let old_path =
        std::path::PathBuf::from(path);

    let parent = old_path
        .parent()
        .ok_or_else(|| {
            "file has no parent directory"
                .to_owned()
        })?;

    let new_path =
        parent.join(new_name);

    if new_path.exists() {
        return Err(
            "a file with that name already exists"
                .to_owned()
        );
    }

    std::fs::rename(
        &old_path,
        &new_path,
    )
    .map_err(|error| {
        format!(
            "failed to rename file: {error}"
        )
    })?;

    Ok(
        new_path
            .to_string_lossy()
            .into_owned()
    )
}

#[tauri::command]
fn emit_all(
    source: String,
) -> Result<EmitResult, String> {
    let timestamp =
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

    let filename =
        format!(
            "tint-{}-{timestamp}.prim",
            process::id(),
        );

    let source_path =
        std::env::temp_dir()
            .join(filename);

    fs::write(
        &source_path,
        source,
    )
    .map_err(|error| {
        format!(
            "failed to create temporary Primer source: {error}"
        )
    })?;

    let result = (|| {
        run_primer(
            "check",
            &source_path,
        )?;

        let c =
            run_primer(
                "emit-c",
                &source_path,
            )?;

        let llvm =
            run_primer(
                "emit-llvm",
                &source_path,
            )?;

        let wat =
            run_primer(
                "emit-wat",
                &source_path,
            )?;

        let qbe =
            run_primer(
                "emit-qbe",
                &source_path,
            )?;

        let qbe_asm =
            run_qbe_asm(
                &qbe,
                timestamp,
            )
            .unwrap_or_else(
                |error| {
                    format!(
                        "QBE ASM unavailable:\n{error}"
                    )
                },
            );

        Ok(EmitResult {
            c,
            llvm,
            wat,
            qbe,
            qbe_asm,
        })
    })();

    let _ =
        fs::remove_file(
            &source_path,
        );

    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                emit_all,
                rename_source,
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running Tint");
}
