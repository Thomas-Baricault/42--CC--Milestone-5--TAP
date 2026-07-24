#[derive(
    Clone,
    Eq,
    Hash,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    strum_macros::EnumIter,
)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    East,
    North,
    South,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::East => Self::West,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

impl std::str::FromStr for Direction {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "east" => Ok(Self::East),
            "north" => Ok(Self::North),
            "south" => Ok(Self::South),
            "west" => Ok(Self::West),
            _ => Err(crate::utils::invalid_input(&format!("invalid direction '{s}'"))),
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::East => write!(f, "east"),
            Self::North => write!(f, "north"),
            Self::South => write!(f, "south"),
            Self::West => write!(f, "west"),
        }
    }
}
