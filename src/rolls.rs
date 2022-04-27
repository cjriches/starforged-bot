use std::cmp::{min, Ordering};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use rand::Rng;

use crate::{InputType, OutputType};

/// The outcome of an action or progress roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionRoll {
    pub action_die: OutputType,
    pub bonus: Option<InputType>,
    pub challenge_dice: [OutputType; 2],
}

impl ActionRoll {
    /// Generate a random action roll.
    pub fn random(bonus: impl Into<Option<InputType>>) -> Self {
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
    pub fn score(&self) -> Option<OutputType> {
        Some(min(self.action_die + u32::from(self.bonus?), 10))
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
                "***Action Roll: [{}]+{} = {} vs [{}] [{}] ({}{})***",
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
                "***Action Roll: [{}] vs [{}] [{}]{}***",
                self.action_die,
                self.challenge_dice[0],
                self.challenge_dice[1],
                if self.is_match() { " (Match)" } else { "" }
            )
        }
    }
}

/// The result of a progress roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressRoll {
    pub bonus: Option<InputType>,
    pub challenge_dice: [OutputType; 2],
}

impl ProgressRoll {
    /// Generate a random progress roll.
    pub fn random(bonus: impl Into<Option<InputType>>) -> Self {
        let mut rng = rand::thread_rng();
        let challenge_dice = [rng.gen_range(1..=10), rng.gen_range(1..=10)];
        Self {
            bonus: bonus.into(),
            challenge_dice,
        }
    }

    /// What is the total score of this roll?
    /// Only known if the bonus is known.
    pub fn score(&self) -> Option<OutputType> {
        Some(min(self.bonus?.into(), 10))
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleRoll {
    pub outcomes: Vec<OutputType>,
}

impl OracleRoll {
    /// Generate a random oracle roll.
    pub fn random(num: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut outcomes = Vec::with_capacity(num);
        for _ in 0..num {
            outcomes.push(rng.gen_range(1..=100));
        }
        Self { outcomes }
    }
}

impl Display for OracleRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = vec!["Oracle Roll:".to_string()];
        for outcome in &self.outcomes {
            string.push(format!(" [{}]", outcome));
        }

        write!(f, "***{}***", string.join(""))
    }
}

/// The specification for a custom roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollSpec {
    pub dice: Vec<InputType>,
    pub bonuses: Vec<InputType>,
}

impl FromStr for RollSpec {
    type Err = ();

    /// Parse a `RollSpec` from a string like `3d6+5`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse_roll_spec::parse(s)
    }
}

/// A die with a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RolledDie {
    pub size: OutputType,
    pub roll: OutputType,
}

/// The result of a custom roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomRoll {
    /// All rolled dice, in descending order of size.
    pub rolls: Vec<RolledDie>,
    /// The total bonus.
    pub bonus: OutputType,
}

impl CustomRoll {
    /// Perform a custom roll.
    pub fn random(spec: RollSpec) -> Self {
        let mut rng = rand::thread_rng();
        let mut rolls = Vec::new();
        for die in spec.dice {
            let roll = rng.gen_range(1..=die).into();
            rolls.push(RolledDie {
                size: die.into(),
                roll,
            });
        }
        rolls.sort_unstable_by(|a, b| b.size.cmp(&a.size));
        let bonus = spec.bonuses.into_iter().map(Into::<OutputType>::into).sum();
        Self { rolls, bonus }
    }

    /// Get the list of all dice in this roll, e.g. `[2d4, 1d6, 5d8]`.
    /// This is returned as a list of `(count, size)` pairs.
    /// We depend on the invariant that `self.rolls` is in descending size order.
    pub fn dice(&self) -> Vec<(OutputType, OutputType)> {
        let mut dice = Vec::new();
        let mut size = OutputType::MAX;
        for die in &self.rolls {
            match die.size.cmp(&size) {
                Ordering::Less => {
                    dice.push((1, die.size));
                    size = die.size;
                }
                Ordering::Equal => {
                    let current = &mut dice.last_mut().unwrap().0;
                    *current += 1;
                }
                Ordering::Greater => {
                    panic!("self.rolls was not in descending order!");
                }
            }
        }
        dice
    }
}

impl Display for CustomRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = vec!["Roll".to_string()];

        // Assemble string representing the roll.
        for (count, size) in self.dice() {
            string.push(format!(" {}d{}", count, size));
            string.push(" +".to_string());
        }

        // Add the bonus if nonzero.
        if self.bonus > 0 {
            string.push(format!(" {}", self.bonus));
        } else {
            // Remove the trailing " +".
            string.pop().unwrap();
        }
        string.push(": ".to_string());

        // Add the results.
        let mut total = 0;
        for roll in &self.rolls {
            total += roll.roll;
            string.push(format!(" [{}]", roll.roll));
        }

        // Add the bonus.
        if self.bonus > 0 {
            total += self.bonus;
            string.push(format!(" + {}", self.bonus));
        }

        // Add the total (only if there was more than one contributor).
        if self.rolls.len() > 1 || self.bonus > 0 {
            string.push(format!("  (Total: {})", total));
        }

        write!(f, "***{}***", string.join(""))
    }
}
