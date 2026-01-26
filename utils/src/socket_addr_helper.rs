use std::net::SocketAddr;
use tokio::net::lookup_host;

/// Helper function to parse a string into a SocketAddr with optional default port
pub async fn get_socket_addr(
    address_str: impl AsRef<str>,
    default_port: u16,
) -> Result<SocketAddr, std::io::Error> {
    let address = address_str.as_ref().trim();

    // Return error if input is empty after trimming
    if address.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Empty address string",
        ));
    }

    // Check if the address contains a port
    if let Some((host, port_str)) = parse_host_and_port(address) {
        let port = port_str.parse::<u16>().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid port number '{}': {}", port_str, e),
            )
        })?;

        return resolve_to_socket_addr(host, port).await;
    }

    // No port specified, use default port
    resolve_to_socket_addr(address, default_port).await
}

/// Parse host and port from address string
fn parse_host_and_port(address: &str) -> Option<(&str, &str)> {
    if address.starts_with('[')
        && let Some(close_bracket) = address.find(']')
        && close_bracket + 1 < address.len()
        && address.as_bytes()[close_bracket + 1] == b':'
    {
        let host = &address[1..close_bracket];
        let port = &address[close_bracket + 2..];
        return Some((host, port));
    }

    // Handle IPv4 addresses and hostnames with ports
    if let Some(colon_pos) = address.rfind(':') {
        // Check if this is not part of an IPv6 address without brackets
        if !address.contains('[') && !address.contains(']') {
            let host = &address[..colon_pos];
            let port = &address[colon_pos + 1..];

            // Basic validation to avoid false positives
            if !host.is_empty() && !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
                return Some((host, port));
            }
        }
    }

    None
}

/// Resolve host to SocketAddr, handling both IP addresses and domain names
async fn resolve_to_socket_addr(host: &str, port: u16) -> Result<SocketAddr, std::io::Error> {
    // First try to parse as IP address (IPv4 or IPv6)
    if let Ok(ip_addr) = host.parse() {
        return Ok(SocketAddr::new(ip_addr, port));
    }

    // If it's not a valid IP address, treat it as a domain name and perform DNS lookup
    let lookup_addr = format!("{}:{}", host, port);
    let mut addrs = lookup_host(&lookup_addr).await?;

    if let Some(addr) = addrs.next() {
        Ok(addr)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Could not resolve host '{}'", host),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ipv4_with_port() {
        let result = get_socket_addr("127.0.0.1:8080", 80).await;
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_ipv4_without_port() {
        let result = get_socket_addr("192.168.1.1", 443).await;
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 443);
        assert_eq!(addr.ip().to_string(), "192.168.1.1");
    }

    #[tokio::test]
    async fn test_ipv6_with_port() {
        let result = get_socket_addr("[::1]:8080", 80).await;
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "::1");
    }

    #[tokio::test]
    async fn test_ipv6_without_port() {
        let result = get_socket_addr("[::1]", 443).await;
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 443);
        assert_eq!(addr.ip().to_string(), "::1");
    }

    #[tokio::test]
    async fn test_invalid_port() {
        let result = get_socket_addr("127.0.0.1:99999", 80).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_string() {
        let result = get_socket_addr("", 80).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whitespace_trimming() {
        let result = get_socket_addr("  127.0.0.1:8080  ", 80).await;
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 8080);
    }

    #[tokio::test]
    async fn test_domain_name_with_port() {
        // This test will only pass if localhost resolves
        let result = get_socket_addr("localhost:8080", 80).await;
        if result.is_ok() {
            let addr = result.unwrap();
            assert_eq!(addr.port(), 8080);
            // localhost should resolve to 127.0.0.1 or ::1
            assert!(addr.ip().is_loopback());
        }
    }

    #[tokio::test]
    async fn test_domain_name_without_port() {
        // This test will only pass if localhost resolves
        let result = get_socket_addr("localhost", 443).await;
        if result.is_ok() {
            let addr = result.unwrap();
            assert_eq!(addr.port(), 443);
            // localhost should resolve to 127.0.0.1 or ::1
            assert!(addr.ip().is_loopback());
        }
    }

    #[tokio::test]
    async fn test_parse_host_and_port() {
        // IPv4 with port
        assert_eq!(
            parse_host_and_port("192.168.1.1:8080"),
            Some(("192.168.1.1", "8080"))
        );

        // IPv6 with port
        assert_eq!(parse_host_and_port("[::1]:8080"), Some(("::1", "8080")));

        // Hostname with port
        assert_eq!(
            parse_host_and_port("example.com:443"),
            Some(("example.com", "443"))
        );

        // No port
        assert_eq!(parse_host_and_port("192.168.1.1"), None);
        assert_eq!(parse_host_and_port("example.com"), None);

        // Invalid cases
        assert_eq!(parse_host_and_port(":"), None);
        assert_eq!(parse_host_and_port("192.168.1.1:"), None);
    }
}
