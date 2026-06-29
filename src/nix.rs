pub(crate) fn nix_to_io(err: nix::Error) -> std::io::Error {
    std::io::Error::from_raw_os_error(err as i32)
}
