use std::borrow::Cow;

use character_converter::CharacterConverter;
use once_cell::sync::Lazy;

use super::Normalizer;
use crate::detection::{Language, Script};
use crate::Token;

/// Normalize Chinese characters by converting them into Simplified Chinese characters.
///
/// This Normalizer uses [`character_converter`] internally to normalize the provided token.
pub struct ChineseNormalizer;

impl Normalizer for ChineseNormalizer {
    fn normalize<'o>(&self, mut token: Token<'o>) -> Box<dyn Iterator<Item = Token<'o>> + 'o> {
        if CONVERTER.is_traditional(token.lemma()) {
            token.lemma = Cow::Owned(CONVERTER.traditional_to_simplified(token.lemma()));
        }

        Box::new(Some(token).into_iter())
    }

    fn should_normalize(&self, script: Script, _language: Option<Language>) -> bool {
        script == Script::Cj
    }
}

static CONVERTER: Lazy<CharacterConverter> = Lazy::new(CharacterConverter::new);

#[cfg(test)]
mod test {
    use std::borrow::Cow::Owned;

    use crate::normalizer::test::test_normalizer;

    // base tokens to normalize.
    fn tokens() -> Vec<Token<'static>> {
        vec![
            Token {
                lemma: Owned("尊嚴".to_string()),
                char_end: 2,
                byte_end: 6,
                script: Script::Cj,
                ..Default::default()
            },
            Token {
                lemma: Owned("生而自由".to_string()),
                char_end: 4,
                byte_end: 12,
                script: Script::Cj,
                ..Default::default()
            },
        ]
    }

    // expected result of the current Normalizer.
    fn normalizer_result() -> Vec<Token<'static>> {
        vec![
            Token {
                // lowercased
                lemma: Owned("尊严".to_string()),
                char_end: 2,
                byte_end: 6,
                script: Script::Cj,
                ..Default::default()
            },
            Token {
                // lowercased
                lemma: Owned("生而自由".to_string()),
                char_end: 4,
                byte_end: 12,
                script: Script::Cj,
                ..Default::default()
            },
        ]
    }

    // expected result of the complete Normalizer pieline.
    fn normalized_tokens() -> Vec<Token<'static>> {
        vec![
            Token {
                lemma: Owned("尊严".to_string()),
                char_end: 2,
                byte_end: 6,
                script: Script::Cj,
                ..Default::default()
            },
            Token {
                lemma: Owned("生而自由".to_string()),
                char_end: 4,
                byte_end: 12,
                script: Script::Cj,
                ..Default::default()
            },
        ]
    }

    test_normalizer!(ChineseNormalizer, tokens(), normalizer_result(), normalized_tokens());
}
