use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    WindowsX64,
    LinuxX64,
}

impl Target {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "x86_64-pc-windows-msvc" => Ok(Self::WindowsX64),

            "x86_64-unknown-linux-gnu" => Ok(Self::LinuxX64),

            _ => Err(format!("unsupported target: {value}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::WindowsX64 => "x86_64-pc-windows-msvc",

            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
        }
    }

    pub fn object_extension(self) -> &'static str {
        match self {
            Self::WindowsX64 => "obj",
            Self::LinuxX64 => "o",
        }
    }

    pub fn qbe_target(self) -> &'static str {
        match self {
            Self::WindowsX64 => "amd64_win",
            Self::LinuxX64 => "amd64_sysv",
        }
    }
}

pub struct ObserveRequest {
    pub source: String,
    pub source_path: Option<String>,
    pub target: Target,
    pub annotate_origins: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub sources: String,
    pub ir: String,
    pub mir: String,
    pub ssa: String,

    pub c: String,
    pub c_asm: String,

    pub llvm: String,
    pub llvm_asm: String,

    pub wat: String,

    pub qbe: String,
    pub qbe_asm: String,

    pub direct_asm: String,
    pub object: String,

    pub bytecode: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPath {
    Ir,
    Mir,
    Ssa,
    Vm,
}

impl ExecutionPath {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "ir" => Ok(Self::Ir),
            "mir" => Ok(Self::Mir),
            "ssa" => Ok(Self::Ssa),
            "vm" => Ok(Self::Vm),

            _ => Err(format!("unsupported execution path: {value}")),
        }
    }
}

pub struct ExecuteRequest {
    pub source: String,
    pub source_path: Option<String>,
    pub path: ExecutionPath,
}

#[derive(Serialize)]
pub struct ExecutionResult {
    pub output: String,
}
