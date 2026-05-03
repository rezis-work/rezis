//! Environment-backed configuration: `.env` (via [`dotenvy`]) and **`PORT`**.

/// Minimal runtime config from process environment (v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RezisConfig {
    pub port: u16,
}

fn parse_port(raw: Option<String>) -> u16 {
    raw.and_then(|s| s.parse().ok()).unwrap_or(3000)
}

impl RezisConfig {
    /// Loads `.env` from the current directory (ignored if missing), then reads **`PORT`**.
    /// Invalid or missing **`PORT`** defaults to **3000**.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            port: parse_port(std::env::var("PORT").ok()),
        }
    }

    /// Listen address on all IPv4 interfaces (`0.0.0.0`) and [`Self::port`].
    pub fn bind_address(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_port_defaults_and_valid() {
        assert_eq!(parse_port(None), 3000);
        assert_eq!(parse_port(Some(String::new())), 3000);
        assert_eq!(parse_port(Some("not-a-port".into())), 3000);
        assert_eq!(parse_port(Some("8080".into())), 8080);
    }

    #[test]
    fn bind_address_format() {
        let c = RezisConfig { port: 3000 };
        assert_eq!(c.bind_address(), "0.0.0.0:3000");
    }
}
