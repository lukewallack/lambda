use crate::either::Either;

// There are two kinds of Parsers: wrapped and unwrapped.
// Wrapped Parsers are stored in a Box.
// Unwrapped Parsers are not.
// All parser combinators work on WrappedParsers.
// So, the unwrapped Parsers that we supply here must be wrapped by calling
// Box::new(...) prior to their use in a parser combinator.

// A Parser<T> is a wrapped Parser that maps strings to values of type T and the
// unparsed portion of the string.
// When parsing fails, the Parser returns None.
pub type Parser<T> = Box<dyn Fn(&str) -> Option<(T, &str)>>;

/// An unwrapped Parser that consumes all whitespace characters.
/// Parsing fails when there is not matching whitespace.
pub fn whitespace(input: &str) -> Option<(String, &str)> {
    let mut idx = 0;
    for ch in input.chars() {
        if ch.is_whitespace() {
            idx += 1;
        } else {
            break;
        }
    }

    if idx == 0 {
        return None;
    }

    let (whitespace, rest) = input.split_at(idx);
    return Some((whitespace.to_string(), rest));
}

/// An unwrapped parser that matches an identifier. An identifier matches the
/// regular expression (a-bA-Z)+(a-bA-Z0-9)*.
/// This handles any whitespace prior to the identifier.
/// It returns None if no identifier is able to be parsed.
pub fn identifier(input: &str) -> Option<(String, &str)> {
    let (_, input) = kleene_star(Box::new(whitespace))(input).unwrap();

    let mut idx = 0;
    for ch in input.chars() {
        if idx == 0 && !ch.is_alphabetic() {
            // Error: We must start with an alphabetic character.
            return None;
        } else if !ch.is_alphanumeric() {
            // We can only contain alphanumeric characters.
            break;
        }
        idx += 1;
    }

    if idx == 0 {
        return None;
    }

    let (symbol, rest) = input.split_at(idx);
    return Some((symbol.to_string(), rest));
}

/// An unwrapped parser that consumes any whitespace, and then checks that the
/// remaining string contains a literal match of `str`.
pub fn chars(str: String) -> Parser<String> {
    Box::new(move |input: &str| {
        let (_, input) = kleene_star(Box::new(whitespace))(input).unwrap();

        if input.starts_with(&str) {
            let (_, rest) = input.split_at(str.len());
            return Some((str.to_string(), rest));
        } else {
            return None;
        }
    })
}

/// A parser combinator that applies its parameter `p` 0 or more times.
/// This never returns None.
/// The return value is a vector that contains each of the objects that `p`
/// parsed.
pub fn kleene_star<T: 'static>(p: Parser<T>) -> Parser<Vec<T>> {
    Box::new(move |input: &str| {
        let mut result = Vec::new();
        let mut data = input;

        while let Some((r, next)) = p(data) {
            result.push(r);
            data = next;
        }

        Some((result, data))
    })
}

/// A parser combinator that applies `p1` first and then `p2` on the remaining
/// unparsed string.
/// If either fail to parse, then None is returned.
/// The return value is a tuple containing the results of applying both parsers.
pub fn and<T1: 'static, T2: 'static>(p1: Parser<T1>, p2: Parser<T2>) -> Parser<(T1, T2)> {
    Box::new(move |input: &str| {
        let (r1, rest1) = p1(input)?;
        let (r2, rest2) = p2(rest1)?;
        Some(((r1, r2), rest2))
    })
}

/// A parser combinator that applies `p1` first. If `p1` succeeds, then it
/// returns the object that `p1` parsed. If `p1` fails, then it applies `p2`.
/// If `p2` succeeds, then `p2`'s return object is returned. Otherwise, None is
/// returned.
pub fn or<T1: 'static, T2: 'static>(p1: Parser<T1>, p2: Parser<T2>) -> Parser<Either<T1, T2>> {
    Box::new(move |input: &str| {
        match p1(input) {
            Some((r1, rest1)) => Some((Either::Left(r1), rest1)),
            None => match p2(input) {
                Some((r2, rest2)) => Some((Either::Right(r2), rest2)),
                None => None,
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace(""), None);
        assert_eq!(whitespace("  "), Some(("  ".to_string(), "")));
        assert_eq!(whitespace("  foo"), Some(("  ".to_string(), "foo")));
        assert_eq!(
            whitespace("  \n\tfoo "),
            Some(("  \n\t".to_string(), "foo "))
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(identifier(""), None);
        assert_eq!(identifier("123 "), None);
        assert_eq!(identifier("foo"), Some(("foo".to_string(), "")));
        assert_eq!(identifier(" foo"), Some(("foo".to_string(), "")));
        assert_eq!(identifier(" foo bar"), Some(("foo".to_string(), " bar")));
    }

    #[test]
    fn test_chars() {
        assert_eq!(chars("foo".to_string())(""), None);
        assert_eq!(
            chars("foo".to_string())("foo"),
            Some(("foo".to_string(), ""))
        );
        assert_eq!(chars("foo".to_string())("bar"), None,);
        assert_eq!(
            chars("foo".to_string())("foo bar"),
            Some(("foo".to_string(), " bar")),
        );
        assert_eq!(
            chars("foo".to_string())("   \tfoo bar"),
            Some(("foo".to_string(), " bar")),
        );
    }

    #[test]
    fn test_and() {
        let parser = and(chars("foo".to_string()), chars("bar".to_string()));
        assert_eq!(
            parser("foobar"),
            Some((("foo".to_string(), "bar".to_string()), ""))
        );

        assert_eq!(
            parser("foobar baz"),
            Some((("foo".to_string(), "bar".to_string()), " baz"))
        );

        assert_eq!(parser("barfoo"), None);
        assert_eq!(parser("foobaz"), None);
        assert_eq!(parser(""), None);
    }

    #[test]
    fn test_kleene_star() {
        let parser = kleene_star(chars("foo".to_string()));
        assert_eq!(parser("bar"), Some((vec![], "bar")));
        assert_eq!(parser(""), Some((vec![], "")));
        assert_eq!(parser("foo"), Some((vec!["foo".to_string()], "")));
        assert_eq!(
            parser("foofoo"),
            Some((vec!["foo".to_string(), "foo".to_string()], ""))
        );
        assert_eq!(
            parser("foofoofoo bar"),
            Some((
                vec!["foo".to_string(), "foo".to_string(), "foo".to_string()],
                " bar"
            ))
        );
        assert_eq!(
            parser("foo  foo  bar"),
            Some((vec!["foo".to_string(), "foo".to_string()], "  bar"))
        );
    }

    #[test]
    fn test_or() {
        let parser = or(chars("foo".to_string()), chars("bar".to_string()));
        // Case 1: p1 succeeds, return p1 parse object
        assert_eq!(
            parser("foo"), 
            Some((Either::Left("foo".to_string()), ""))
        );
        assert_eq!(
            parser("foo bar"),
            Some((Either::Left("foo".to_string()), " bar"))
        );

        // Case 2: p1 fails, p2 succeeds
        assert_eq!(
            parser("bar"), 
            Some((Either::Right("bar".to_string()), ""))
        );
        assert_eq!(
            parser("bar baz"),
            Some((Either::Right("bar".to_string()), " baz"))
        );

        // Case 3: p1 & p2 fail
        assert_eq!(parser("baz"), None);
        assert_eq!(parser(""), None);
    }
}
