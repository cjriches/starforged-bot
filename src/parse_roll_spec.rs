use logos::{Lexer, Logos};

use crate::RollSpec;

/// A token that we use to parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
enum Token {
    /// Single or multi-digit numbers.
    #[regex(r"\d+", |lex| lex.slice().parse())]
    Number(u32),

    /// The 'd' in '2d4+5' (case-insensitive).
    #[regex(r"d|D")]
    D,

    /// The '+' in '2d4+5'.
    #[token("+")]
    Plus,

    /// Catch-all for anything else.
    #[error]
    Error,
}

/// Parse a `RollSpec` from a string slice.
pub fn parse(input: &str) -> Result<RollSpec, ()> {
    let mut lex: Lexer<Token> = Token::lexer(input);
    let mut current: Token;
    macro_rules! next {
        () => {{
            current = lex.next().ok_or(())?;
            current
        }};
    }

    // Parse the number of dice, which may be absent meaning 1.
    let num_dice = if let Token::Number(n) = next!() {
        next!(); // Ensure `current` now points to what should be "d".
        n
    } else {
        1
    };

    // Ensure we now have a d.
    if let Token::D = current {
        // Correct.
    } else {
        return Err(());
    }

    // Parse the dice size, which must be present.
    let dice_size = if let Token::Number(n) = next!() {
        n
    } else {
        return Err(());
    };

    // Parse the bonus, which is optional.
    let mut bonus = 0;
    while let Some(Token::Plus) = lex.next() {
        if let Token::Number(n) = next!() {
            bonus += n;
        } else {
            return Err(());
        }
    }

    // Ensure there are no more tokens.
    if lex.next().is_some() {
        return Err(());
    }

    Ok(RollSpec {
        num_dice,
        dice_size,
        bonus,
    })
}
