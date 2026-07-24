#[derive(
    Clone,
    Default,
)]
pub struct Response {
    pub payload: crate::messages::Payload,
}

impl std::str::FromStr for Response {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut message = s.to_string();
        if !crate::messages::utils::parse_begin(&mut message, "OK") {
            return Err(crate::utils::invalid_input(&format!("invalid response '{s}'")));
        }
        Ok(Self {
            payload: match crate::messages::utils::parse_payload(&mut message) {
                Ok(v) => v,
                Err(_) => return Err(crate::utils::invalid_input(&format!("invalid response '{s}'"))),
            }
        })
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::messages::utils::write_vec(f, vec![
            "OK".to_string(),
            self.payload.to_string(),
        ])
    }
}
