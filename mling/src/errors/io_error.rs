use mingling::{macros::{group, r_println, renderer}, res::ResExitCode};

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
pub fn render_error_io(err: ErrorIo, ec: &mut ResExitCode) {
    match err.kind() {
        std::io::ErrorKind::NotFound => {
            r_println!("{}", eformat_cargo!("file or directory not found"));
            ec.exit_code = EC_IO_ERR_NOT_FOUND;
        }
        std::io::ErrorKind::PermissionDenied => {
            r_println!("{}", eformat_cargo!("permission denied"));
            ec.exit_code = EC_IO_ERR_PERMISSION_DENIED;
        }
        std::io::ErrorKind::ConnectionRefused => {
            r_println!("{}", eformat_cargo!("connection refused"));
            ec.exit_code = EC_IO_ERR_CONNECTION_REFUSED;
        }
        std::io::ErrorKind::ConnectionReset => {
            r_println!("{}", eformat_cargo!("connection reset"));
            ec.exit_code = EC_IO_ERR_CONNECTION_RESET;
        }
        std::io::ErrorKind::HostUnreachable => {
            r_println!("{}", eformat_cargo!("host unreachable"));
            ec.exit_code = EC_IO_ERR_HOST_UNREACHABLE;
        }
        std::io::ErrorKind::NetworkUnreachable => {
            r_println!("{}", eformat_cargo!("network unreachable"));
            ec.exit_code = EC_IO_ERR_NETWORK_UNREACHABLE;
        }
        std::io::ErrorKind::ConnectionAborted => {
            r_println!("{}", eformat_cargo!("connection aborted"));
            ec.exit_code = EC_IO_ERR_CONNECTION_ABORTED;
        }
        std::io::ErrorKind::NotConnected => {
            r_println!("{}", eformat_cargo!("not connected"));
            ec.exit_code = EC_IO_ERR_NOT_CONNECTED;
        }
        std::io::ErrorKind::AddrInUse => {
            r_println!("{}", eformat_cargo!("address in use"));
            ec.exit_code = EC_IO_ERR_ADDR_IN_USE;
        }
        std::io::ErrorKind::AddrNotAvailable => {
            r_println!("{}", eformat_cargo!("address not available"));
            ec.exit_code = EC_IO_ERR_ADDR_NOT_AVAILABLE;
        }
        std::io::ErrorKind::NetworkDown => {
            r_println!("{}", eformat_cargo!("network down"));
            ec.exit_code = EC_IO_ERR_NETWORK_DOWN;
        }
        std::io::ErrorKind::BrokenPipe => {
            r_println!("{}", eformat_cargo!("broken pipe"));
            ec.exit_code = EC_IO_ERR_BROKEN_PIPE;
        }
        std::io::ErrorKind::AlreadyExists => {
            r_println!("{}", eformat_cargo!("file or directory already exists"));
            ec.exit_code = EC_IO_ERR_ALREADY_EXISTS;
        }
        std::io::ErrorKind::WouldBlock => {
            r_println!("{}", eformat_cargo!("operation would block"));
            ec.exit_code = EC_IO_ERR_WOULD_BLOCK;
        }
        std::io::ErrorKind::NotADirectory => {
            r_println!("{}", eformat_cargo!("not a directory"));
            ec.exit_code = EC_IO_ERR_NOT_A_DIRECTORY;
        }
        std::io::ErrorKind::IsADirectory => {
            r_println!("{}", eformat_cargo!("is a directory"));
            ec.exit_code = EC_IO_ERR_IS_A_DIRECTORY;
        }
        std::io::ErrorKind::DirectoryNotEmpty => {
            r_println!("{}", eformat_cargo!("directory not empty"));
            ec.exit_code = EC_IO_ERR_DIRECTORY_NOT_EMPTY;
        }
        std::io::ErrorKind::ReadOnlyFilesystem => {
            r_println!("{}", eformat_cargo!("read-only filesystem"));
            ec.exit_code = EC_IO_ERR_READ_ONLY_FILESYSTEM;
        }
        std::io::ErrorKind::StaleNetworkFileHandle => {
            r_println!("{}", eformat_cargo!("stale network file handle"));
            ec.exit_code = EC_IO_ERR_STALE_NETWORK_FILE_HANDLE;
        }
        std::io::ErrorKind::InvalidInput => {
            r_println!("{}", eformat_cargo!("invalid input"));
            ec.exit_code = EC_IO_ERR_INVALID_INPUT;
        }
        std::io::ErrorKind::InvalidData => {
            r_println!("{}", eformat_cargo!("invalid data"));
            ec.exit_code = EC_IO_ERR_INVALID_DATA;
        }
        std::io::ErrorKind::TimedOut => {
            r_println!("{}", eformat_cargo!("timed out"));
            ec.exit_code = EC_IO_ERR_TIMED_OUT;
        }
        std::io::ErrorKind::WriteZero => {
            r_println!("{}", eformat_cargo!("write zero"));
            ec.exit_code = EC_IO_ERR_WRITE_ZERO;
        }
        std::io::ErrorKind::StorageFull => {
            r_println!("{}", eformat_cargo!("storage full"));
            ec.exit_code = EC_IO_ERR_STORAGE_FULL;
        }
        std::io::ErrorKind::NotSeekable => {
            r_println!("{}", eformat_cargo!("not seekable"));
            ec.exit_code = EC_IO_ERR_NOT_SEEKABLE;
        }
        std::io::ErrorKind::QuotaExceeded => {
            r_println!("{}", eformat_cargo!("quota exceeded"));
            ec.exit_code = EC_IO_ERR_QUOTA_EXCEEDED;
        }
        std::io::ErrorKind::FileTooLarge => {
            r_println!("{}", eformat_cargo!("file too large"));
            ec.exit_code = EC_IO_ERR_FILE_TOO_LARGE;
        }
        std::io::ErrorKind::ResourceBusy => {
            r_println!("{}", eformat_cargo!("resource busy"));
            ec.exit_code = EC_IO_ERR_RESOURCE_BUSY;
        }
        std::io::ErrorKind::ExecutableFileBusy => {
            r_println!("{}", eformat_cargo!("executable file busy"));
            ec.exit_code = EC_IO_ERR_EXECUTABLE_FILE_BUSY;
        }
        std::io::ErrorKind::Deadlock => {
            r_println!("{}", eformat_cargo!("deadlock"));
            ec.exit_code = EC_IO_ERR_DEADLOCK;
        }
        std::io::ErrorKind::CrossesDevices => {
            r_println!("{}", eformat_cargo!("crosses devices"));
            ec.exit_code = EC_IO_ERR_CROSSES_DEVICES;
        }
        std::io::ErrorKind::TooManyLinks => {
            r_println!("{}", eformat_cargo!("too many links"));
            ec.exit_code = EC_IO_ERR_TOO_MANY_LINKS;
        }
        std::io::ErrorKind::InvalidFilename => {
            r_println!("{}", eformat_cargo!("invalid filename"));
            ec.exit_code = EC_IO_ERR_INVALID_FILENAME;
        }
        std::io::ErrorKind::ArgumentListTooLong => {
            r_println!("{}", eformat_cargo!("argument list too long"));
            ec.exit_code = EC_IO_ERR_ARGUMENT_LIST_TOO_LONG;
        }
        std::io::ErrorKind::Interrupted => {
            r_println!("{}", eformat_cargo!("interrupted"));
            ec.exit_code = EC_IO_ERR_INTERRUPTED;
        }
        std::io::ErrorKind::Unsupported => {
            r_println!("{}", eformat_cargo!("unsupported"));
            ec.exit_code = EC_IO_ERR_UNSUPPORTED;
        }
        std::io::ErrorKind::UnexpectedEof => {
            r_println!("{}", eformat_cargo!("unexpected end of file"));
            ec.exit_code = EC_IO_ERR_UNEXPECTED_EOF;
        }
        std::io::ErrorKind::OutOfMemory => {
            r_println!("{}", eformat_cargo!("out of memory"));
            ec.exit_code = EC_IO_ERR_OUT_OF_MEMORY;
        }
        std::io::ErrorKind::Other => {
            r_println!("{}", eformat_cargo!(err.to_string()));
            ec.exit_code = EC_IO_ERR_OTHER;
        }
        _ => {
            r_println!("{}", eformat_cargo!(err.to_string()));
            ec.exit_code = EC_IO_ERR_OTHER;
        }
    }
}
