//! Native System Call Standard Library
//! 
//! This module provides low-level system operations for native Hint programs:
//! - File I/O (read, write, open, close)
//! - Network I/O (TCP/UDP sockets)
//! - Process management
//! - Environment variables
//! - Time/date operations
//! - Path manipulation
//! 
//! These functions work directly with system calls, no C library required.

pub mod fs;
pub mod net;
pub mod process;
pub mod env;
pub mod time;
pub mod path;
pub mod syscalls;

pub use fs::{File, FileHandle, FileMode, FileSystem};
pub use net::{Socket, TcpListener, TcpStream, UdpSocket, Address};
pub use process::{Process, ProcessBuilder, ExitCode};
pub use env::Environment;
pub use time::{Time, Duration, Instant};
pub use path::{Path, PathBuf};
pub use syscalls::SyscallResult;

use crate::stdlib::{StdlibFunction, StdlibImpl, IntrinsicId};
use crate::semantics::HintType;

/// Initialize native stdlib functions
pub fn init() -> Vec<StdlibFunction> {
    let mut functions = Vec::new();
    
    // File System functions
    functions.extend(fs::init());
    
    // Network functions
    functions.extend(net::init());
    
    // Process functions
    functions.extend(process::init());
    
    // Environment functions
    functions.extend(env::init());
    
    // Time functions
    functions.extend(time::init());
    
    // Path functions
    functions.extend(path::init());
    
    functions
}

/// Native intrinsic function IDs
#[derive(Debug, Clone, Copy)]
pub enum NativeIntrinsic {
    // File System
    FileOpen,
    FileRead,
    FileWrite,
    FileClose,
    FileSeek,
    FileFlush,
    FileMetadata,
    FileRemove,
    FileRename,
    FileCopy,
    DirCreate,
    DirRead,
    DirRemove,
    CurrentDir,
    SetCurrentDir,
    
    // Network
    TcpConnect,
    TcpBind,
    TcpListen,
    TcpAccept,
    TcpRead,
    TcpWrite,
    UdpBind,
    UdpSend,
    UdpRecv,
    SocketClose,
    
    // Process
    ProcessSpawn,
    ProcessWait,
    ProcessKill,
    ProcessId,
    ProcessExit,
    CurrentExe,
    
    // Environment
    EnvGet,
    EnvSet,
    EnvRemove,
    EnvVars,
    
    // Time
    TimeNow,
    TimeSleep,
    TimeUnix,
    
    // Path
    PathJoin,
    PathDirname,
    PathBasename,
    PathExtension,
    PathIsAbsolute,
    PathExists,
    PathIsFile,
    PathIsDir,
}

impl NativeIntrinsic {
    pub fn to_stdlib_function(&self) -> StdlibFunction {
        match self {
            // File System
            NativeIntrinsic::FileOpen => StdlibFunction {
                name: "file_open".to_string(),
                params: vec![HintType::String, HintType::String],
                return_type: HintType::Pointer(Box::new(HintType::Void)),
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeFileOpen),
                description: "Open a file with specified mode",
            },
            NativeIntrinsic::FileRead => StdlibFunction {
                name: "file_read".to_string(),
                params: vec![HintType::Pointer(Box::new(HintType::Void))],
                return_type: HintType::String,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeFileRead),
                description: "Read entire file contents",
            },
            NativeIntrinsic::FileWrite => StdlibFunction {
                name: "file_write".to_string(),
                params: vec![HintType::Pointer(Box::new(HintType::Void)), HintType::String],
                return_type: HintType::Void,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeFileWrite),
                description: "Write string to file",
            },
            NativeIntrinsic::FileClose => StdlibFunction {
                name: "file_close".to_string(),
                params: vec![HintType::Pointer(Box::new(HintType::Void))],
                return_type: HintType::Void,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeFileClose),
                description: "Close open file handle",
            },
            
            // Network
            NativeIntrinsic::TcpConnect => StdlibFunction {
                name: "tcp_connect".to_string(),
                params: vec![HintType::String, HintType::Int(crate::semantics::IntSize::I32)],
                return_type: HintType::Pointer(Box::new(HintType::Void)),
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeTcpConnect),
                description: "Connect to TCP server",
            },
            NativeIntrinsic::TcpRead => StdlibFunction {
                name: "tcp_read".to_string(),
                params: vec![HintType::Pointer(Box::new(HintType::Void))],
                return_type: HintType::String,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeTcpRead),
                description: "Read from TCP socket",
            },
            NativeIntrinsic::TcpWrite => StdlibFunction {
                name: "tcp_write".to_string(),
                params: vec![HintType::Pointer(Box::new(HintType::Void)), HintType::String],
                return_type: HintType::Int(crate::semantics::IntSize::I64),
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeTcpWrite),
                description: "Write to TCP socket",
            },
            
            // Process
            NativeIntrinsic::ProcessSpawn => StdlibFunction {
                name: "spawn".to_string(),
                params: vec![HintType::String],
                return_type: HintType::Pointer(Box::new(HintType::Void)),
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeProcessSpawn),
                description: "Spawn new process",
            },
            NativeIntrinsic::ProcessWait => StdlibFunction {
                name: "wait".to_string(),
                params: vec![HintType::Pointer(Box::new(HintType::Void))],
                return_type: HintType::Int(crate::semantics::IntSize::I32),
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeProcessWait),
                description: "Wait for process to exit",
            },
            
            // Environment
            NativeIntrinsic::EnvGet => StdlibFunction {
                name: "env_get".to_string(),
                params: vec![HintType::String],
                return_type: HintType::String,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeEnvGet),
                description: "Get environment variable",
            },
            NativeIntrinsic::EnvSet => StdlibFunction {
                name: "env_set".to_string(),
                params: vec![HintType::String, HintType::String],
                return_type: HintType::Void,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeEnvSet),
                description: "Set environment variable",
            },
            
            // Time
            NativeIntrinsic::TimeNow => StdlibFunction {
                name: "time_now".to_string(),
                params: vec![],
                return_type: HintType::Int(crate::semantics::IntSize::I64),
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeTimeNow),
                description: "Get current timestamp",
            },
            NativeIntrinsic::TimeSleep => StdlibFunction {
                name: "sleep".to_string(),
                params: vec![HintType::Int(crate::semantics::IntSize::I64)],
                return_type: HintType::Void,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeTimeSleep),
                description: "Sleep for milliseconds",
            },
            
            // Path
            NativeIntrinsic::PathExists => StdlibFunction {
                name: "path_exists".to_string(),
                params: vec![HintType::String],
                return_type: HintType::Bool,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativePathExists),
                description: "Check if path exists",
            },
            
            _ => StdlibFunction {
                name: "unknown".to_string(),
                params: vec![],
                return_type: HintType::Void,
                implementation: StdlibImpl::Intrinsic(IntrinsicId::NativeUnknown),
                description: "Unknown native function",
            },
        }
    }
}

/// Get all native stdlib functions
pub fn get_native_stdlib() -> Vec<StdlibFunction> {
    init()
}
