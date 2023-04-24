//! The C# implementation uses inheritance to do this.
//! More specifically, the lexer generated by ANTLR derives from the `IndentAwareLexer`
//! directly, and the `IndentAwareLexer` derives from the ANTLR Lexer base class.
//! Instead of this, we use a proxy/wrapper around the generated lexer to handle everything correctly.
//! TODO: Decide if we want to hide the generated lexer to make sure no one accidentially uses it.

mod collections;

use collections::*;

use std::collections::VecDeque;

use antlr_rust::{
    char_stream::CharStream,
    token::CommonToken,
    token_factory::{CommonTokenFactory, TokenFactory},
    Lexer, TokenSource,
};

use super::generated::yarnspinnerlexer::{self, LocalTokenFactory, YarnSpinnerLexer};

/// A Lexer subclass that detects newlines and generates indent and dedent tokens accordingly.
pub struct IndentAwareYarnSpinnerLexer<
    'input,
    Input: CharStream<From<'input>>,
    TF: TokenFactory<'input> = CommonTokenFactory,
> {
    base: YarnSpinnerLexer<'input, Input>, // TODO: needed?
    pub token: Option<TF::Tok>,
    hit_eof: bool,
    last_token: Option<TF::Tok>,
    pending_tokens: VecDeque<TF::Tok>,
    line_contains_shortcut: bool,
    last_indent: isize,
    unbalanced_indents: VecDeque<isize>,
    last_seen_option_content: Option<isize>,
}

impl<'input, Input: CharStream<From<'input>> + std::ops::Deref> std::ops::Deref
    for IndentAwareYarnSpinnerLexer<'input, Input>
{
    type Target = YarnSpinnerLexer<'input, Input>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

// better_any::tid! {IndentAwareYarnSpinnerLexer} // TODO: needed?

impl<'input, Input: CharStream<From<'input>>> TokenSource<'input>
    for IndentAwareYarnSpinnerLexer<'input, Input>
{
    type TF = CommonTokenFactory; // TODO: correct?

    fn next_token(&mut self) -> <Self::TF as antlr_rust::token_factory::TokenFactory<'input>>::Tok {
        if self.hit_eof && self.pending_tokens.len() > 0 {
            // We have hit the EOF, but we have tokens still pending.
            // Start returning those tokens.
            self.pending_tokens.pop_front(); // TODO: I think that's right?
            todo!()
        } else if self.base.input().size() == 0 {
            self.hit_eof = true;
            Box::new(CommonToken {
                token_type: antlr_rust::token::TOKEN_EOF,
                channel: 0, // See CommonToken.ctor(int, string) in Antlr for C#
                start: 0,   // TODO: does that work? and all after this one as well.
                stop: 0,
                token_index: 0.into(),
                line: 0,
                column: 0,
                text: "<EOF>".into(),
                read_only: true,
            })
        } else {
            // Get the next token, which will enqueue one or more new
            // tokens into the pending tokens queue.
            self.check_next_token();

            if !self.pending_tokens.is_empty() {
                return self.pending_tokens.pop_front().unwrap().to_owned();
            }

            todo!() // C# returns null?!
        }
    }

    fn get_input_stream(&mut self) -> Option<&mut dyn antlr_rust::int_stream::IntStream> {
        self.base.get_input_stream()
    }

    fn get_source_name(&self) -> String {
        self.base.get_source_name()
    }

    fn get_token_factory(&self) -> &'input Self::TF {
        self.base.get_token_factory()
    }
}

/// Copied from generated/yarnspinnerlexer.rs
type From<'a> = <LocalTokenFactory<'a> as TokenFactory<'a>>::From;

impl<'input, Input: CharStream<From<'input>>> IndentAwareYarnSpinnerLexer<'input, Input>
where
    &'input LocalTokenFactory<'input>: Default,
{
    pub fn new(input: Input) -> Self {
        IndentAwareYarnSpinnerLexer {
            // TODO: is that correct? Is ::new sufficient whithout the LocalTokenFactory as param?
            base: YarnSpinnerLexer::new_with_token_factory(
                input,
                <&LocalTokenFactory<'input> as Default>::default(),
            ),
            token: Default::default(), // TODO: correct?
            hit_eof: false,
            last_token: Default::default(),
            pending_tokens: Default::default(),
            line_contains_shortcut: false,
            last_indent: Default::default(),
            unbalanced_indents: Default::default(),
            last_seen_option_content: None,
        }
    }

    fn check_next_token(&mut self) {
        let current = self.base.next_token();

        match current.token_type {
            // Insert indents or dedents depending on the next token's
            // indentation, and enqueues the newline at the correct place
            yarnspinnerlexer::NEWLINE => self.handle_newline_token(current.clone()),
            // Insert dedents before the end of the file, and then
            // enqueues the EOF.
            antlr_rust::token::TOKEN_EOF => self.handle_eof_token(current.clone()),
            yarnspinnerlexer::SHORTCUT_ARROW => {
                self.pending_tokens.push_back(current.clone()); // TODO: check if push_back is correctly modeling this.pendingTokens.Enqueue(currentToken);
                self.line_contains_shortcut = true;
            }
            // we are at the end of the node
            // depth no longer matters
            // clear the stack
            yarnspinnerlexer::BODY_END => {
                // TODO: put those into a well-named function
                self.line_contains_shortcut = false;
                self.last_indent = 0;
                self.unbalanced_indents.clear();
                self.last_seen_option_content = None;
                // [sic from the original!] TODO: this should be empty by now actually...
                self.pending_tokens.push_back(current.clone());
            }
            _ => self.pending_tokens.push_back(current.clone()),
        }

        // TODO: but... really?
        self.last_token = Some(current);
    }

    fn handle_newline_token(
        &self,
        current: Box<antlr_rust::token::GenericToken<std::borrow::Cow<str>>>,
    ) {
        todo!()
    }

    fn handle_eof_token(
        &self,
        current: Box<antlr_rust::token::GenericToken<std::borrow::Cow<str>>>,
    ) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use antlr_rust::{
        common_token_stream::CommonTokenStream, int_stream::IntStream, token::TOKEN_EOF,
        InputStream,
    };

    use crate::prelude::generated::yarnspinnerlexer::{YarnSpinnerLexer, DEDENT, INDENT};

    use super::*;

    const MINIMAL_INPUT: &str = "title: Minimal Yarn
---
This is the one and only line
===";

    #[test]
    fn behaves_like_lexer_for_unindented_input() {
        let generated_lexer = YarnSpinnerLexer::new(InputStream::new(MINIMAL_INPUT));
        let indent_aware_lexer = IndentAwareYarnSpinnerLexer::new(InputStream::new(MINIMAL_INPUT));

        let mut reference_token_stream = CommonTokenStream::new(generated_lexer);
        let mut indent_aware_token_stream = CommonTokenStream::new(indent_aware_lexer);

        assert_eq!(
            reference_token_stream.size(),
            indent_aware_token_stream.size()
        );

        // Sanity check: Make sure at least one token is read: We do have input.
        assert_eq!(
            reference_token_stream.iter().next(),
            indent_aware_token_stream.iter().next()
        );

        // Can not do this, as trying to read EOF panics...
        // Iterator::eq(
        //     reference_token_stream.iter(),
        //     indent_aware_token_stream.iter(),
        // );

        while reference_token_stream.la(1) != TOKEN_EOF {
            assert_eq!(
                reference_token_stream.iter().next(),
                indent_aware_token_stream.iter().next()
            );
        }

        assert_eq!(TOKEN_EOF, reference_token_stream.la(1));
        assert_eq!(TOKEN_EOF, indent_aware_token_stream.la(1));
    }

    #[test]
    fn correctly_indents_and_dedents_with_token() {
        let option_indentation_relevant_input: &str = &("title: Start
---
-> Option 1
    Nice.
-> Option 2
    Nicer
".to_owned() +
    "    " /* Bug when saving in VSCode (maybe even with rustfmt):
    the spaces on an empty line in a string are removed...  */ + &"
    But this belongs to it!

    And this doesn't
===
        ".as_ref());

        let indent_aware_lexer =
            IndentAwareYarnSpinnerLexer::new(InputStream::new(option_indentation_relevant_input));

        let mut indent_aware_token_stream = CommonTokenStream::new(indent_aware_lexer);

        let mut tokens = vec![indent_aware_token_stream.iter().next().unwrap()];

        while indent_aware_token_stream.la(1) != TOKEN_EOF {
            tokens.push(indent_aware_token_stream.iter().next().unwrap());
        }

        assert!(tokens.contains(&INDENT));
        assert!(tokens.contains(&DEDENT));

        // TODO: actually test the order
    }
}
