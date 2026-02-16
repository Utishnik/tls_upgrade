pub enum TlsVersion {
    /// TLS 1.0
    ///
    /// Should only be used when trying to support legacy
    /// SMTP servers that haven't updated to
    /// at least TLS 1.2 yet.
    ///
    /// Supported by `native-tls` and `boring-tls`.
    Tlsv10,
    /// TLS 1.1
    ///
    /// Should only be used when trying to support legacy
    /// SMTP servers that haven't updated to
    /// at least TLS 1.2 yet.
    ///
    /// Supported by `native-tls` and `boring-tls`.
    Tlsv11,
    /// TLS 1.2
    ///
    /// A good option for most SMTP servers.
    ///
    /// Supported by all TLS backends.
    Tlsv12,
    /// TLS 1.3
    ///
    /// The most secure option, although not supported by all SMTP servers.
    ///
    /// Although it is technically supported by all TLS backends,
    /// trying to set it for `native-tls` will give a runtime error.
    Tlsv13,
}