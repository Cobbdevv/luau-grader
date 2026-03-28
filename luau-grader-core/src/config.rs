use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Tier {
    Beginner,
    Intermediate,
    Advanced,
    FrontPage,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Beginner => "beginner",
            Tier::Intermediate => "intermediate",
            Tier::Advanced => "advanced",
            Tier::FrontPage => "front_page",
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "beginner" => Ok(Tier::Beginner),
            "intermediate" => Ok(Tier::Intermediate),
            "advanced" => Ok(Tier::Advanced),
            "front_page" | "frontpage" => Ok(Tier::FrontPage),
            other => Err(format!("unknown tier: {other}")),
        }
    }
}
