use mingling::{
    RenderResult,
    macros::{group, renderer},
    res::ResExitCode,
};
use std::io::Write as _;

use crate::eformat_cargo;

group!(ErrorIo = std::io::Error);

// Error code constants for each std::io::ErrorKind variant
pub const EC_IO_ERR_NOT_FOUND: i32 = 1000;
pub const EC_IO_ERR_PERMISSION_DENIED: i32 = 1001;
pub const EC_IO_ERR_CONNECTION_REFUSED: i32 = 1002;
pub const EC_IO_ERR_CONNECTION_RESET: i32 = 1003;
pub const EC_IO_ERR_HOST_UNREACHABLE: i32 = 1004;
pub const EC_IO_ERR_NETWORK_UNREACHABLE: i32 = 1005;
pub const EC_IO_ERR_CONNECTION_ABORTED: i32 = 1006;
pub const EC_IO_ERR_NOT_CONNECTED: i32 = 1007;
pub const EC_IO_ERR_ADDR_IN_USE: i32 = 1008;
pub const EC_IO_ERR_ADDR_NOT_AVAILABLE: i32 = 1009;
pub const EC_IO_ERR_NETWORK_DOWN: i32 = 1010;
pub const EC_IO_ERR_BROKEN_PIPE: i32 = 1011;
pub const EC_IO_ERR_ALREADY_EXISTS: i32 = 1012;
pub const EC_IO_ERR_WOULD_BLOCK: i32 = 1013;
pub const EC_IO_ERR_NOT_A_DIRECTORY: i32 = 1014;
pub const EC_IO_ERR_IS_A_DIRECTORY: i32 = 1015;
pub const EC_IO_ERR_DIRECTORY_NOT_EMPTY: i32 = 1016;
pub const EC_IO_ERR_READ_ONLY_FILESYSTEM: i32 = 1017;
pub const EC_IO_ERR_STALE_NETWORK_FILE_HANDLE: i32 = 1018;
pub const EC_IO_ERR_INVALID_INPUT: i32 = 1019;
pub const EC_IO_ERR_INVALID_DATA: i32 = 1020;
pub const EC_IO_ERR_TIMED_OUT: i32 = 1021;
pub const EC_IO_ERR_WRITE_ZERO: i32 = 1022;
pub const EC_IO_ERR_STORAGE_FULL: i32 = 1023;
pub const EC_IO_ERR_NOT_SEEKABLE: i32 = 1024;
pub const EC_IO_ERR_QUOTA_EXCEEDED: i32 = 1025;
pub const EC_IO_ERR_FILE_TOO_LARGE: i32 = 1026;
pub const EC_IO_ERR_RESOURCE_BUSY: i32 = 1027;
pub const EC_IO_ERR_EXECUTABLE_FILE_BUSY: i32 = 1028;
pub const EC_IO_ERR_DEADLOCK: i32 = 1029;
pub const EC_IO_ERR_CROSSES_DEVICES: i32 = 1030;
pub const EC_IO_ERR_TOO_MANY_LINKS: i32 = 1031;
pub const EC_IO_ERR_INVALID_FILENAME: i32 = 1032;
pub const EC_IO_ERR_ARGUMENT_LIST_TOO_LONG: i32 = 1033;
pub const EC_IO_ERR_INTERRUPTED: i32 = 1034;
pub const EC_IO_ERR_UNSUPPORTED: i32 = 1035;
pub const EC_IO_ERR_UNEXPECTED_EOF: i32 = 1036;
pub const EC_IO_ERR_OUT_OF_MEMORY: i32 = 1037;
pub const EC_IO_ERR_OTHER: i32 = 1038;

#[renderer]
pub fn render_error_io(err: ErrorIo, ec: &mut ResExitCode) -> RenderResult {
    let mut result = RenderResult::default();
    match err.kind() {
        std::io::ErrorKind::NotFound => {
            writeln!(result, "{}", eformat_cargo!("file or directory not found")).ok();
            ec.exit_code = EC_IO_ERR_NOT_FOUND;
        }
        std::io::ErrorKind::PermissionDenied => {
            writeln!(result, "{}", eformat_cargo!("permission denied")).ok();
            ec.exit_code = EC_IO_ERR_PERMISSION_DENIED;
        }
        std::io::ErrorKind::ConnectionRefused => {
            writeln!(result, "{}", eformat_cargo!("connection refused")).ok();
            ec.exit_code = EC_IO_ERR_CONNECTION_REFUSED;
        }
        std::io::ErrorKind::ConnectionReset => {
            writeln!(result, "{}", eformat_cargo!("connection reset")).ok();
            ec.exit_code = EC_IO_ERR_CONNECTION_RESET;
        }
        std::io::ErrorKind::HostUnreachable => {
            writeln!(result, "{}", eformat_cargo!("host unreachable")).ok();
            ec.exit_code = EC_IO_ERR_HOST_UNREACHABLE;
        }
        std::io::ErrorKind::NetworkUnreachable => {
            writeln!(result, "{}", eformat_cargo!("network unreachable")).ok();
            ec.exit_code = EC_IO_ERR_NETWORK_UNREACHABLE;
        }
        std::io::ErrorKind::ConnectionAborted => {
            writeln!(result, "{}", eformat_cargo!("connection aborted")).ok();
            ec.exit_code = EC_IO_ERR_CONNECTION_ABORTED;
        }
        std::io::ErrorKind::NotConnected => {
            writeln!(result, "{}", eformat_cargo!("not connected")).ok();
            ec.exit_code = EC_IO_ERR_NOT_CONNECTED;
        }
        std::io::ErrorKind::AddrInUse => {
            writeln!(result, "{}", eformat_cargo!("address in use")).ok();
            ec.exit_code = EC_IO_ERR_ADDR_IN_USE;
        }
        std::io::ErrorKind::AddrNotAvailable => {
            writeln!(result, "{}", eformat_cargo!("address not available")).ok();
            ec.exit_code = EC_IO_ERR_ADDR_NOT_AVAILABLE;
        }
        std::io::ErrorKind::NetworkDown => {
            writeln!(result, "{}", eformat_cargo!("network down")).ok();
            ec.exit_code = EC_IO_ERR_NETWORK_DOWN;
        }
        std::io::ErrorKind::BrokenPipe => {
            writeln!(result, "{}", eformat_cargo!("broken pipe")).ok();
            ec.exit_code = EC_IO_ERR_BROKEN_PIPE;
        }
        std::io::ErrorKind::AlreadyExists => {
            writeln!(
                result,
                "{}",
                eformat_cargo!("file or directory already exists")
            )
            .ok();
            ec.exit_code = EC_IO_ERR_ALREADY_EXISTS;
        }
        std::io::ErrorKind::WouldBlock => {
            writeln!(result, "{}", eformat_cargo!("operation would block")).ok();
            ec.exit_code = EC_IO_ERR_WOULD_BLOCK;
        }
        std::io::ErrorKind::NotADirectory => {
            writeln!(result, "{}", eformat_cargo!("not a directory")).ok();
            ec.exit_code = EC_IO_ERR_NOT_A_DIRECTORY;
        }
        std::io::ErrorKind::IsADirectory => {
            writeln!(result, "{}", eformat_cargo!("is a directory")).ok();
            ec.exit_code = EC_IO_ERR_IS_A_DIRECTORY;
        }
        std::io::ErrorKind::DirectoryNotEmpty => {
            writeln!(result, "{}", eformat_cargo!("directory not empty")).ok();
            ec.exit_code = EC_IO_ERR_DIRECTORY_NOT_EMPTY;
        }
        std::io::ErrorKind::ReadOnlyFilesystem => {
            writeln!(result, "{}", eformat_cargo!("read-only filesystem")).ok();
            ec.exit_code = EC_IO_ERR_READ_ONLY_FILESYSTEM;
        }
        std::io::ErrorKind::StaleNetworkFileHandle => {
            writeln!(result, "{}", eformat_cargo!("stale network file handle")).ok();
            ec.exit_code = EC_IO_ERR_STALE_NETWORK_FILE_HANDLE;
        }
        std::io::ErrorKind::InvalidInput => {
            writeln!(result, "{}", eformat_cargo!("invalid input")).ok();
            ec.exit_code = EC_IO_ERR_INVALID_INPUT;
        }
        std::io::ErrorKind::InvalidData => {
            writeln!(result, "{}", eformat_cargo!("invalid data")).ok();
            ec.exit_code = EC_IO_ERR_INVALID_DATA;
        }
        std::io::ErrorKind::TimedOut => {
            writeln!(result, "{}", eformat_cargo!("timed out")).ok();
            ec.exit_code = EC_IO_ERR_TIMED_OUT;
        }
        std::io::ErrorKind::WriteZero => {
            writeln!(result, "{}", eformat_cargo!("write zero")).ok();
            ec.exit_code = EC_IO_ERR_WRITE_ZERO;
        }
        std::io::ErrorKind::StorageFull => {
            writeln!(result, "{}", eformat_cargo!("storage full")).ok();
            ec.exit_code = EC_IO_ERR_STORAGE_FULL;
        }
        std::io::ErrorKind::NotSeekable => {
            writeln!(result, "{}", eformat_cargo!("not seekable")).ok();
            ec.exit_code = EC_IO_ERR_NOT_SEEKABLE;
        }
        std::io::ErrorKind::QuotaExceeded => {
            writeln!(result, "{}", eformat_cargo!("quota exceeded")).ok();
            ec.exit_code = EC_IO_ERR_QUOTA_EXCEEDED;
        }
        std::io::ErrorKind::FileTooLarge => {
            writeln!(result, "{}", eformat_cargo!("file too large")).ok();
            ec.exit_code = EC_IO_ERR_FILE_TOO_LARGE;
        }
        std::io::ErrorKind::ResourceBusy => {
            writeln!(result, "{}", eformat_cargo!("resource busy")).ok();
            ec.exit_code = EC_IO_ERR_RESOURCE_BUSY;
        }
        std::io::ErrorKind::ExecutableFileBusy => {
            writeln!(result, "{}", eformat_cargo!("executable file busy")).ok();
            ec.exit_code = EC_IO_ERR_EXECUTABLE_FILE_BUSY;
        }
        std::io::ErrorKind::Deadlock => {
            writeln!(result, "{}", eformat_cargo!("deadlock")).ok();
            ec.exit_code = EC_IO_ERR_DEADLOCK;
        }
        std::io::ErrorKind::CrossesDevices => {
            writeln!(result, "{}", eformat_cargo!("crosses devices")).ok();
            ec.exit_code = EC_IO_ERR_CROSSES_DEVICES;
        }
        std::io::ErrorKind::TooManyLinks => {
            writeln!(result, "{}", eformat_cargo!("too many links")).ok();
            ec.exit_code = EC_IO_ERR_TOO_MANY_LINKS;
        }
        std::io::ErrorKind::InvalidFilename => {
            writeln!(result, "{}", eformat_cargo!("invalid filename")).ok();
            ec.exit_code = EC_IO_ERR_INVALID_FILENAME;
        }
        std::io::ErrorKind::ArgumentListTooLong => {
            writeln!(result, "{}", eformat_cargo!("argument list too long")).ok();
            ec.exit_code = EC_IO_ERR_ARGUMENT_LIST_TOO_LONG;
        }
        std::io::ErrorKind::Interrupted => {
            writeln!(result, "{}", eformat_cargo!("interrupted")).ok();
            ec.exit_code = EC_IO_ERR_INTERRUPTED;
        }
        std::io::ErrorKind::Unsupported => {
            writeln!(result, "{}", eformat_cargo!("unsupported")).ok();
            ec.exit_code = EC_IO_ERR_UNSUPPORTED;
        }
        std::io::ErrorKind::UnexpectedEof => {
            writeln!(result, "{}", eformat_cargo!("unexpected end of file")).ok();
            ec.exit_code = EC_IO_ERR_UNEXPECTED_EOF;
        }
        std::io::ErrorKind::OutOfMemory => {
            writeln!(result, "{}", eformat_cargo!("out of memory")).ok();
            ec.exit_code = EC_IO_ERR_OUT_OF_MEMORY;
        }
        std::io::ErrorKind::Other => {
            writeln!(result, "{}", eformat_cargo!(err.to_string())).ok();
            ec.exit_code = EC_IO_ERR_OTHER;
        }
        _ => {
            writeln!(result, "{}", eformat_cargo!(err.to_string())).ok();
            ec.exit_code = EC_IO_ERR_OTHER;
        }
    }
    result
}
