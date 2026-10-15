
# PERSONAL NOTES, IGNORE BELOW
## Deliverables
- parse.rs/or
- term.rs/normal_order_reduction
- main.rs/application_parser
- tests

## Parsers and combinators notes
A parser is a program that transforms an input string into a resulting syntax tree, and the unparsed 
remainder of the string.

`type parser (t : Type) = string -> option (t, string)`
`t` is a type param that stands in for the data type representing our syntax tree.
For our \ calculus, a parser has the type `parser term`.
The final result of parsing is stored in an `option` type that holds either `Some (x,y)` when parsing succeeds with the syntax tree `x` and remaining string `y`, or `None` when parsing fails.

A parser combinator is a program that combines two parsers.
In parse.rs, there are several parser combinators. Make sure you understand how each works.

Note, there are 2 kinds of Parsers: wrapped and unwrapped. Wrapped Parsers are stored in a `Box`, unwrapped Parsers are not.
All parser combinators work on WrappedParsers. Unwrapped Parsers supplied must be wrapped by calling `Box::new(...)` prior to their use in a combinator.

`pub type Parser<T> = Box<dyn Fn(&str) -> Option<(T, &str)>>;`

A `Parser<T>` is a wrapped Parser thnat maps strings to values of type `T` and unparsed portion of the string.
When parsing fails, the Parser returns `None`.

### `and` parser combinator:
- Takes parsers `p1`, `p2`
- First applies `p1`, and then applies `p2` on the remaining unparsed string.
- If either fail top parse, None is returned
- Return value is a tuple containing results of applying both parsers

### `kleene_star` parser combinator:
- Takes parser `p`
- Applies `p` 0 or more times
- Never returns `None`
- Return value = vector containing each of the objects that `p` parsed

### `or` parser combinator
- Takes parsers `p1`, `p2`
- Applies `p1` first. If `p1` succeeds, return the object `p1` parsed.
- If `p1` fails, applies `p2`. If `p2` succeeds, return `p2`'s return object.
- Otherwise `None` is returned.

### `chars` parser
- Takes String `str`
- Unwrapped parser, consumes any whitespace & checks that remaining string contains a literal match of `str`

### `identifier` parser
- Unwrapped parser that matches an identifier.
- Identifier matches the regex `(a-bA-Z)+(a-bA-Z0-9)*`.
- Handles any whitespace prior to identifier
- Returns `None` if no identifier is able to be parsed.

### `whitespace` parser
- Unwrapped parser that consumes all whitespace chars.
- Parsing fails when there is not matching whitespace.
    `if idx == 0 return None;`