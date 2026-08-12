use serde::Serialize;
use std::{
    fs,
    path::Path,
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
struct EmitResult {
    c: String,
    llvm: String,
    wat: String,
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
fn emit_all(source: String) -> Result<EmitResult, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let filename = format!("tint-{}-{timestamp}.prim", process::id(),);

    let source_path = std::env::temp_dir().join(filename);

    fs::write(&source_path, source)
        .map_err(|error| format!("failed to create temporary Primer source: {error}"))?;

    let result = (|| {
        run_primer("check", &source_path)?;

        let c = run_primer("emit-c", &source_path)?;

        let llvm = run_primer("emit-llvm", &source_path)?;

        let wat = run_primer("emit-wat", &source_path)?;

        Ok(EmitResult { c, llvm, wat })
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
        .invoke_handler(
            tauri::generate_handler![
                emit_all,
                rename_source,
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running Tint");
}
