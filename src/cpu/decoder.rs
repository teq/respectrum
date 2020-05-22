use super::tokens::*;

/// CPU opcode decoder
pub struct Decoder {

    prefix: Option<PrefixCode>,

}

impl Decoder {

    /// Reset internal state
    pub fn reset(&mut self) {
        self.prefix = None;
    }

    /// Process next byte and produce CPU token
    pub fn process(&mut self, byte: u8) -> Token {

        let token = match byte {
            0xcb => Token::Prefix { code: PrefixCode::CB },
            0xed => Token::Prefix { code: PrefixCode::ED },
            0xdd => Token::Prefix { code: PrefixCode::DD },
            0xfd => Token::Prefix { code: PrefixCode::CB },
            _ => panic!()
        };

        if let Token::Prefix { code } = token {
            self.prefix = Some(code);
        }

        return token;

    }

}
