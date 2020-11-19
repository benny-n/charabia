use std::borrow::Cow;

use jieba_rs::Jieba as JiebaTokenizer;

use crate::{Token, TokenKind};
use crate::processors::ProcessedText;
use super::{InternalTokenizer, TokenStream};

pub struct Jieba {
    jieba: JiebaTokenizer,
}

impl InternalTokenizer for Jieba {
    fn tokenize<'a>(&self, s: &'a ProcessedText<'a>) -> TokenStream<'a> {
        let tokenized = self.jieba.tokenize(&s.processed, jieba_rs::TokenizeMode::Default, false);

        let original_byte_len = s.original.len();
        let mut original = s.original.char_indices()
            // map only byte indices
            .map(|(byte_index, _)| byte_index)
            // add ending byte index
            .chain(std::iter::once(original_byte_len));

        TokenStream {
            inner: Box::new(tokenized.into_iter().scan(0, move |byte_index, jieba_token| {
                let char_start = jieba_token.start;
                let char_end = jieba_token.end;
                let byte_start = *byte_index;

                // iter.nth(0) == iter.next(), so nth is computed as `char_end - char_start - 1`
                // but not for the first iteration where nth is computed as `char_end`
                let byte_end = match *byte_index {
                    0 => original.nth(char_end),
                    _ => original.nth(char_end - char_start - 1),
                };

                #[cfg(test)]
                let byte_end = byte_end.unwrap();

                #[cfg(not(test))]
                let byte_end = match byte_end {
                    Some(byte_end) => byte_end,
                    None => return None
                };

                *byte_index = byte_end;

                Some(Token {
                    kind: TokenKind::Word,
                    word: Cow::Borrowed(jieba_token.word),
                    char_index: char_start,
                    byte_start,
                    byte_end,
                })
            }))
        }
    }
}

impl Default for Jieba {
    fn default() -> Self { Jieba { jieba: JiebaTokenizer::new() } }
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::Lazy;

    static JIEBA_TOKENIZER: Lazy<Jieba> = Lazy::new(|| Jieba::default());

    #[test]
    fn test_simple() {
        let orig = "The quick (\"brown\") fox can't jump 32.3 feet, right? Brr, it's 29.3°F!";
        let processed = ProcessedText {
            original: orig,
            processed: Cow::Borrowed(orig),
        };
        let tokens = JIEBA_TOKENIZER.tokenize(&processed).map(|Token { word, .. }| word.to_owned()).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            ["The", " ", "quick", " ", "(", "\"", "brown", "\"", ")", " ", "fox", " ",
            "can", "\'", "t", " ", "jump", " ", "32", ".", "3", " ", "feet", ",", " ",
            "right", "?", " ", "Brr", ",", " ", "it", "\'", "s", " ", "29", ".", "3", "°", "F", "!"]
        );
        
        let orig = "為一包含一千多萬目詞的帶標記平衡語料庫";
        let processed = ProcessedText {
            original: orig,
            processed: Cow::Borrowed(orig),
        };
        let tokens = JIEBA_TOKENIZER.tokenize(&processed).map(|Token { word, .. }| word.to_owned()).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            ["為", "一", "包含", "一千多", "萬", "目", "詞", "的", "帶", "標", "記", "平衡", "語", "料", "庫"]
        );
    }

    #[test]
    fn test_byte_positions() {
        let orig = "The quick (\"brown\") fox can't jump 32.3 feet, right? Brr, it's 29.3°F!";
        let processed = ProcessedText {
            original: orig,
            processed: Cow::Borrowed(orig),
        };
        let tokens = JIEBA_TOKENIZER.tokenize(&processed);
        assert_eq!(orig, tokens.map(|t| &orig[t.byte_start..t.byte_end]).collect::<String>());
        
        let orig = "為一包含一千多萬目詞的帶標記平衡語料庫";
        let processed = ProcessedText {
            original: orig,
            processed: Cow::Borrowed(orig),
        };
        let tokens = JIEBA_TOKENIZER.tokenize(&processed);
        assert_eq!(orig, tokens.map(|t| &orig[t.byte_start..t.byte_end]).collect::<String>());
    }

    #[test]
    fn test_char_indices() {
        let orig = "The quick (\"brown\") fox can't jump 32.3 feet, right? Brr, it's 29.3°F!";
        let processed = ProcessedText {
            original: orig,
            processed: Cow::Borrowed(orig),
        };
        let positions = JIEBA_TOKENIZER.tokenize(&processed).map(|Token { char_index, .. }| char_index).collect::<Vec<_>>();
        assert_eq!(
            positions,
            [0, 3, 4, 9, 10, 11, 12, 17, 18, 19, 20,
            23, 24, 27, 28, 29, 30, 34, 35, 37, 38, 39,
            40, 44, 45, 46, 51, 52, 53, 56, 57, 58, 60,
            61, 62, 63, 65, 66, 67, 68, 69]
        );

        let orig = "為一包含一千多萬目詞的帶標記平衡語料庫";
        let processed = ProcessedText {
            original: orig,
            processed: Cow::Borrowed(orig),
        };
        let positions = JIEBA_TOKENIZER.tokenize(&processed).map(|Token { char_index, .. }| char_index).collect::<Vec<_>>();
        assert_eq!(
            positions,
            [0, 1, 2, 4, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 18]
        );

    }
}
