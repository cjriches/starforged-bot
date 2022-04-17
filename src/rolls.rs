use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use rand::Rng;

/// The outcome of an action or progress roll.
#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Miss,
    WeakHit,
    StrongHit,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Outcome::Miss => "Miss",
            Outcome::WeakHit => "Weak Hit",
            Outcome::StrongHit => "Strong Hit",
        };
        write!(f, "{}", name)
    }
}

/// The result of an action roll.
#[derive(Debug, Clone, Copy)]
pub struct ActionRoll {
    pub action_die: u32,
    pub bonus: Option<u32>,
    pub challenge_dice: [u32; 2],
}

impl ActionRoll {
    /// Generate a random action roll.
    pub fn random(bonus: impl Into<Option<u32>>) -> Self {
        let mut rng = rand::thread_rng();
        let action_die = rng.gen_range(1..=6);
        let challenge_dice = [rng.gen_range(1..=10), rng.gen_range(1..=10)];
        Self {
            action_die,
            bonus: bonus.into(),
            challenge_dice,
        }
    }

    /// What is the total score of this roll?
    /// Only known if the bonus is known.
    pub fn score(&self) -> Option<u32> {
        Some(min(self.action_die + self.bonus?, 10))
    }

    /// What is the outcome of this roll?
    /// Only known if the bonus is known.
    pub fn outcome(&self) -> Option<Outcome> {
        let score = self.score()?;
        let mut higher_than = 0;
        for challenge in self.challenge_dice {
            if score > challenge {
                higher_than += 1;
            }
        }
        Some(match higher_than {
            0 => Outcome::Miss,
            1 => Outcome::WeakHit,
            2 => Outcome::StrongHit,
            _ => unreachable!(),
        })
    }

    /// Do the challenge dice match?
    pub fn is_match(&self) -> bool {
        self.challenge_dice[0] == self.challenge_dice[1]
    }
}

impl Display for ActionRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(bonus) = self.bonus {
            write!(
                f,
                "***Action Roll: {}+{} = {} vs [{}] [{}] ({}{})***",
                self.action_die,
                bonus,
                self.score().unwrap(),
                self.challenge_dice[0],
                self.challenge_dice[1],
                if self.is_match() { "Matched " } else { "" },
                self.outcome().unwrap()
            )
        } else {
            write!(
                f,
                "***Action Roll: {} vs [{}] [{}]{}***",
                self.action_die,
                self.challenge_dice[0],
                self.challenge_dice[1],
                if self.is_match() { " (Match)" } else { "" }
            )
        }
    }
}

/// The result of a progress roll.
#[derive(Debug, Clone, Copy)]
pub struct ProgressRoll {
    pub bonus: Option<u32>,
    pub challenge_dice: [u32; 2],
}

impl ProgressRoll {
    /// Generate a random progress roll.
    pub fn random(bonus: impl Into<Option<u32>>) -> Self {
        let mut rng = rand::thread_rng();
        let challenge_dice = [rng.gen_range(1..=10), rng.gen_range(1..=10)];
        Self {
            bonus: bonus.into(),
            challenge_dice,
        }
    }

    /// What is the total score of this roll?
    /// Only known if the bonus is known.
    pub fn score(&self) -> Option<u32> {
        Some(min(self.bonus?, 10))
    }

    /// What is the outcome of this roll?
    /// Only known if the bonus is known.
    pub fn outcome(&self) -> Option<Outcome> {
        let score = self.score()?;
        let mut higher_than = 0;
        for challenge in self.challenge_dice {
            if score > challenge {
                higher_than += 1;
            }
        }
        Some(match higher_than {
            0 => Outcome::Miss,
            1 => Outcome::WeakHit,
            2 => Outcome::StrongHit,
            _ => unreachable!(),
        })
    }

    /// Do the challenge dice match?
    pub fn is_match(&self) -> bool {
        self.challenge_dice[0] == self.challenge_dice[1]
    }
}

impl Display for ProgressRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.bonus.is_some() {
            write!(
                f,
                "***Progress Roll: {} vs [{}] [{}] ({}{})***",
                self.score().unwrap(),
                self.challenge_dice[0],
                self.challenge_dice[1],
                if self.is_match() { "Matched " } else { "" },
                self.outcome().unwrap()
            )
        } else {
            write!(
                f,
                "***Progress Roll: [{}] [{}]{}***",
                self.challenge_dice[0],
                self.challenge_dice[1],
                if self.is_match() { " (Match)" } else { "" }
            )
        }
    }
}

/// The result of an oracle roll.
#[derive(Debug, Clone, Copy)]
pub struct OracleRoll {
    pub outcome: u32,
}

impl OracleRoll {
    /// Generate a random oracle roll.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let outcome = rng.gen_range(1..=100);
        Self { outcome }
    }
}

impl Display for OracleRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "***Oracle Roll: [{}]***", self.outcome)
    }
}

/// The specification for a custom roll.
#[derive(Debug, Clone, Copy)]
pub struct RollSpec {
    pub num_dice: u32,
    pub dice_size: u32,
    pub bonus: u32,
}

impl FromStr for RollSpec {
    type Err = ();

    /// Parse a `RollSpec` from a string like `3d6+5`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse_roll_spec::parse(s)
    }
}

/// The result of a custom roll.
#[derive(Debug, Clone)]
pub struct CustomRoll {
    pub rolls: Vec<u32>,
    pub bonuses: Vec<u32>,
}

impl CustomRoll {
    /// Perform a custom roll.
    pub fn random(specs: impl IntoIterator<Item = RollSpec>) -> Self {
        let mut rng = rand::thread_rng();
        let mut rolls = Vec::new();
        let mut bonuses = Vec::new();
        for spec in specs {
            if spec.bonus > 0 {
                bonuses.push(spec.bonus);
            }
            for _ in 0..spec.num_dice {
                rolls.push(rng.gen_range(1..=spec.dice_size));
            }
        }
        Self { rolls, bonuses }
    }
}

impl Display for CustomRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = vec!["Custom Roll:".to_string()];
        let mut total = 0;
        for roll in &self.rolls {
            total += roll;
            string.push(format!(" [{}]", roll));
        }
        for bonus in &self.bonuses {
            total += bonus;
            string.push(format!(" +{}", bonus));
        }
        string.push(format!(" Total: {}", total));

        write!(f, "***{}***", string.join(""))
    }
}
