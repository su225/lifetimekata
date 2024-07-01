use std::cmp::max;

use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken<'s> {
    /// This is just text without anything special.
    RawText(&'s str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'s str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

impl<'s> MatcherToken<'s> {
    fn match_string<'x>(&self, input: &'x str) -> Option<&'x str> {
        match self {
            MatcherToken::RawText(ref text_to_match) if input.starts_with(text_to_match) =>
                    Option::Some(&input[..text_to_match.len()]),

            MatcherToken::OneOfText(ref choices) => {
                let mut longest_match = 0;
                for &ch in choices.iter() {
                    if input.starts_with(ch) && ch.len() > longest_match {
                        longest_match = ch.len();
                    }
                }
                if longest_match > 0 {
                    Option::Some(&input[..longest_match])
                } else {
                    Option::None
                }
            },
            MatcherToken::WildCard if !input.is_empty() => {
                let first_char = input.chars().next().unwrap();
                Option::Some(&input[..first_char.len_utf8()])
            },
            _ => Option::None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher<'s> {
    /// This is the actual text of the matcher
    text: &'s str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'s>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

#[derive(Debug)]
enum MatcherPatternParseError {
    WildcardInOneOf,
    RecursiveOneOf,
    PipeNotAllowedInStandalone,
    ClosingParenInStandalone,
    Incomplete,
}

#[derive(Eq, PartialEq)]
enum ParseMode {
    Standalone,
    OneOf,
}

struct PatternParser<'p> {
    pattern: &'p str,
    cur_tok_start: Option<usize>,
}

impl<'p> PatternParser<'p> {
    pub fn new(pattern: &'p str) -> Self {
        PatternParser { pattern: pattern, cur_tok_start: Option::None }
    }

    pub fn parse_into(mut self) -> Result<Vec<MatcherToken<'p>>, MatcherPatternParseError> {
        let mut mat_toks = vec![];
        let mut oneof_choices = vec![];
        let mut parse_mode = ParseMode::Standalone;
        let mut bytes_so_far = 0;
        for c in self.pattern.chars() {
            if c == '.' {
                if parse_mode == ParseMode::OneOf {
                    return Err(MatcherPatternParseError::WildcardInOneOf);
                }
                self.maybe_extract_token(bytes_so_far)
                    .map(|tok| mat_toks.push(MatcherToken::RawText(tok)));
                mat_toks.push(MatcherToken::WildCard);
            } else if c == '(' {
                if parse_mode == ParseMode::OneOf {
                    return Err(MatcherPatternParseError::RecursiveOneOf);
                }
                self.maybe_extract_token(bytes_so_far)
                    .map(|tok| mat_toks.push(MatcherToken::RawText(tok)));
                parse_mode = ParseMode::OneOf;
            } else if c == '|' {
                if parse_mode == ParseMode::Standalone {
                    return Err(MatcherPatternParseError::PipeNotAllowedInStandalone);
                }
                self.maybe_extract_token(bytes_so_far)
                    .map(|tok| oneof_choices.push(tok));
            } else if c == ')' {
                if parse_mode == ParseMode::Standalone {
                    return Err(MatcherPatternParseError::ClosingParenInStandalone);
                }
                self.maybe_extract_token(bytes_so_far)
                    .map(|tok| oneof_choices.push(tok));
                parse_mode = ParseMode::Standalone;
                mat_toks.push(MatcherToken::OneOfText(oneof_choices.clone()));
                oneof_choices.clear();
            } else {
                if self.cur_tok_start.is_none() {
                    self.cur_tok_start = Option::Some(bytes_so_far);
                }
            }
            bytes_so_far += c.len_utf8();
        }
        if parse_mode == ParseMode::OneOf {
            return Err(MatcherPatternParseError::Incomplete)
        }
        self.maybe_extract_token(bytes_so_far)
            .map(|tok| mat_toks.push(MatcherToken::RawText(tok)));
        return Ok(mat_toks);
    }

    fn maybe_extract_token(&mut self, end_idx: usize) -> Option<&'p str> {
        let res = if end_idx == 0 {
            Option::None
        } else if let Option::Some(start_idx) = self.cur_tok_start {
            Option::Some(&self.pattern[start_idx..end_idx])
        } else {
            Option::None
        };
        self.cur_tok_start = Option::None;
        res
    }
}

impl<'s> Matcher<'s> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'s str) -> Option<Matcher<'s>> {
        let pattern = PatternParser::new(&text).parse_into();
        println!("{pattern:?}");
        pattern.ok().map(|tokens| Matcher {
            text,
            tokens,
            most_tokens_matched: 0,
        })
    }

    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    fn match_string<'m>(&mut self, string: &'m str) -> Vec<(&MatcherToken, &'m str)> {
        let mut matched_till = 0;
        let mut match_result = vec![];
        for tok in self.tokens.iter() {
            if let Some(matched) = tok.match_string(&string[matched_till..]) {
                matched_till += matched.len();
                match_result.push((tok, matched))
            } else {
                break;
            }
        }
        self.most_tokens_matched = max(self.most_tokens_matched, match_result.len());
        match_result
    }
}

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};
    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcd💪".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "💪"),
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
