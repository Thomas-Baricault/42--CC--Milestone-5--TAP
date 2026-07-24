pub fn invalid_input(s: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        s,
    )
}
