use logos::{Lexer, Logos};

use crate::{rolls::RollSpec, InputType};

/// Parse the numbers from a slice of `XdY` format.
fn parse_xdy(slice: &str) -> Option<(InputType, InputType)> {
    // Either 'd' or 'D' is guaranteed by the format.
    let (count, size) = slice
        .split_once('d')
        .unwrap_or_else(|| slice.split_once('D').unwrap());
    // The count might be missing.
    let count = if count.is_empty() {
        1
    } else {
        count.parse().ok()?
    };
    // The size must be present.
    let size = size.parse().ok()?;
    Some((count, size))
}

/// A token that we use to parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
enum Token {
    /// An `XdY` roll specification.
    #[regex(r"\d*(d|D)\d+", |lex| parse_xdy(lex.slice()))]
    XdY((InputType, InputType)),

    /// A bonus specification.
    #[regex(r"\d+", |lex| lex.slice().parse())]
    Bonus(InputType),

    /// The `+` character.
    #[token("+")]
    Plus,

    /// Whitespace (ignored)
    #[regex(r"\s+", logos::skip)]
    Whitespace,

    /// Catch-all for anything else.
    #[error]
    Error,
}

/// Parse a `RollSpec` from a string slice.
pub fn parse(input: &str) -> Result<RollSpec, ()> {
    let mut lex: Lexer<Token> = Token::lexer(input);
    let mut dice = Vec::new();
    let mut bonuses = Vec::new();

    while let Some(token) = lex.next() {
        // Get the next die or bonus.
        match token {
            Token::XdY((count, size)) => {
                for _ in 0..count {
                    dice.push(size);
                }
            }
            Token::Bonus(bonus) => {
                bonuses.push(bonus);
            }
            Token::Plus => {
                return Err(());
            }
            Token::Whitespace => {
                unreachable!() // Whitespace tokens should be skipped.
            }
            Token::Error => {
                return Err(());
            }
        }
        // If there are more tokens, we must see a plus before anything else.
        if let Some(token) = lex.next() {
            if token != Token::Plus {
                return Err(());
            }
        }
    }

    // Check we have at least one die.
    if dice.is_empty() {
        return Err(());
    }

    Ok(RollSpec { dice, bonuses })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_die() {
        let spec = parse("d4").unwrap();
        assert_eq!(spec.dice, vec![4]);
        assert!(spec.bonuses.is_empty());
    }

    #[test]
    fn multiple_dice() {
        let spec = parse("4d8").unwrap();
        assert_eq!(spec.dice, vec![8, 8, 8, 8]);
        assert!(spec.bonuses.is_empty());
    }

    #[test]
    fn with_bonus() {
        let spec = parse("1d10 +2").unwrap();
        assert_eq!(spec.dice, vec![10]);
        assert_eq!(spec.bonuses, vec![2]);
    }

    #[test]
    fn multiple_sizes() {
        let spec = parse("2d6+1d4").unwrap();
        assert_eq!(spec.dice, vec![6, 6, 4]);
        assert!(spec.bonuses.is_empty());
    }

    #[test]
    fn multiple_bonuses() {
        let spec = parse("1d12 +2+1+1").unwrap();
        assert_eq!(spec.dice, vec![12]);
        assert_eq!(spec.bonuses, vec![2, 1, 1]);
    }

    #[test]
    fn whitespace() {
        let specs = vec![
            parse("1d4+1d6+2+1").unwrap(),
            parse("1d4 +1d6+2+1").unwrap(),
            parse("1d4 +   1d6+2+1").unwrap(),
            parse("1d4 +   1d6  +2+1").unwrap(),
            parse("1d4 +   1d6  + 2+1").unwrap(),
            parse("1d4 +   1d6  + 2+  1").unwrap(),
            parse("1d4 +   1d6  + 2 +  1").unwrap(),
            parse("1d4 +   1d6  + 2 +  1  ").unwrap(),
            parse(" 1d4 +   1d6  + 2 +  1  ").unwrap(),
        ];
        for i in 0..(specs.len() - 1) {
            assert_eq!(specs[i], specs[i + 1]);
        }
    }

    #[test]
    fn ordering() {
        let spec = parse("1 + d4").unwrap();
        assert_eq!(spec.dice, vec![4]);
        assert_eq!(spec.bonuses, vec![1]);
    }

    #[test]
    fn capitalisation() {
        let spec1 = parse("2d4").unwrap();
        assert_eq!(spec1.dice, vec![4, 4]);
        assert!(spec1.bonuses.is_empty());

        let spec2 = parse("2D4").unwrap();
        assert_eq!(spec1, spec2);
    }

    #[test]
    fn bad_input() {
        parse("").unwrap_err();
        parse("5").unwrap_err();
        parse("+ 8").unwrap_err();
        parse("+d6").unwrap_err();
        parse("1d4 + fish").unwrap_err();
        parse("2d4 ++ 6").unwrap_err();
        parse("2 d4").unwrap_err();
        parse("2d 4").unwrap_err();
        parse("2 d 4").unwrap_err();
        parse("2d4 1d6").unwrap_err();
        parse("300d4").unwrap_err();
        parse("d1000").unwrap_err();
    }
}
