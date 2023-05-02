//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Compiler/Declaration.cs>
//!
//! ## Implementation notes
//!
//! [`Range`] has been replaced with the more idiomatic [`RangeInclusive<Position>`].

use crate::parser_rule_context_ext::ParserRuleContextExt;
use crate::prelude::{ActualTokenStream, Diagnostic};
use antlr_rust::token::Token;
use std::fmt::{Debug, Display};
use std::ops::RangeInclusive;
use yarn_slinger_core::prelude::convertible::Convertible;
use yarn_slinger_core::types::Type;

/// Information about a declaration. Stored inside a declaration table,
/// which is produced from the Compiler.
///
/// You do not create instances of this class yourself. They are
/// generated by the [`Compiler`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Declaration {
    /// The name of this declaration.
    pub name: String,

    /// The default value of this declaration, if no value has been
    /// specified in code or is available from a [`Dialogue`]'s
    /// [`IVariableStorage`].
    pub default_value: Option<Convertible>,

    /// A string describing the purpose of this declaration.
    pub description: Option<String>,

    /// The name of the file in which this declaration was found.
    ///
    /// If this declaration was not found in a Yarn source file, this
    /// will be [`DeclarationSource::External`].
    pub source_file_name: DeclarationSource,

    /// The name of the node in which this declaration was found.
    ///
    /// If this declaration was not found in a Yarn source file, this
    /// will be [`None`].
    pub source_node_name: Option<String>,

    /// A value indicating whether this declaration was implicitly
    /// inferred from usage.
    ///
    /// If `true`, this declaration was implicitly inferred from usage.
    /// If `false`, this declaration appears in the source code.
    pub is_implicit: bool,

    /// The type of the variable, as represented by an object found
    /// in a variant of [`Type`].
    pub r#type: Option<Type>,

    /// The range of text at which this declaration occurs.
    ///
    /// This range refers to the declaration of the symbol itself, and
    /// not any syntax surrounding it. For example, the declaration
    /// `<<declare $x = 1>>` would have a range referring to the `$x`
    /// symbol.
    pub range: Option<RangeInclusive<Position>>,
}

impl Declaration {
    /// Gets the line number at which this Declaration was found in the
    /// source file.
    ///
    /// If this [`Declaration`] was not found in a Yarn source file,
    /// this will be [`None`].
    pub fn source_file_line(&self) -> Option<usize> {
        self.range.as_ref()?.start().line.into()
    }

    pub fn with_type(mut self, r#type: impl Into<Option<Type>>) -> Self {
        self.r#type = r#type.into();
        self
    }

    pub fn with_default_value(mut self, default_value: impl Into<Convertible>) -> Self {
        self.default_value = Some(default_value.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_description_optional(mut self, description: impl Into<Option<String>>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_source_file_name(mut self, source_file_name: impl Into<DeclarationSource>) -> Self {
        self.source_file_name = source_file_name.into();
        self
    }

    pub fn with_source_node_name(mut self, source_node_name: impl Into<String>) -> Self {
        self.source_node_name = Some(source_node_name.into());
        self
    }
    pub fn with_source_node_name_optional(
        mut self,
        source_node_name: impl Into<Option<String>>,
    ) -> Self {
        self.source_node_name = source_node_name.into();
        self
    }

    pub fn with_implicit(mut self) -> Self {
        self.is_implicit = true;
        self
    }

    pub fn with_range(mut self, range: impl Into<RangeInclusive<Position>>) -> Self {
        self.range = Some(range.into());
        self
    }

    pub fn eq(&self, other: &Self, epsilon: f32) -> bool {
        self.name == other.name
            && self.description == other.description
            && self.source_file_name == other.source_file_name
            && self.source_node_name == other.source_node_name
            && self.is_implicit == other.is_implicit
            && self.r#type == other.r#type
            && self.range == other.range
            && match (&self.default_value, &other.default_value) {
                (Some(a), Some(b)) => a.eq(b, epsilon),
                (None, None) => true,
                _ => false,
            }
    }
}

/// The source of a declaration.
///
/// ## Implementation notes
///
/// In the original implementation, [`External`] is just a magic string.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DeclarationSource {
    #[default]
    External,
    File(String),
}

impl From<String> for DeclarationSource {
    fn from(file_name: String) -> Self {
        Self::File(file_name)
    }
}

impl From<&str> for DeclarationSource {
    fn from(file_name: &str) -> Self {
        file_name.to_owned().into()
    }
}

/// Represents a position in a multi-line string.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Position {
    /// The zero-indexed line of this position.
    pub line: usize,

    /// The zero-indexed character number of this position.
    pub character: usize,
}

pub trait RangeSource<'input>: ParserRuleContextExt<'input> {
    fn range(&self, token_stream: &ActualTokenStream<'input>) -> RangeInclusive<Position> {
        let start = Position {
            line: self.start().get_line() as usize,
            character: self.start().get_column() as usize + 1,
        };
        let stop = Position {
            line: self.stop().get_line() as usize,
            character: self.start().get_column() as usize
                + self.get_text_with_whitespace(token_stream).len(),
        };
        start..=stop
    }
}

impl<'input, T: ParserRuleContextExt<'input>> RangeSource<'input> for T {}

impl Display for DeclarationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::External => write!(f, "(External)"),
            Self::File(file_name) => write!(f, "{}", file_name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DeferredTypeDiagnostic {
    pub(crate) name: String,
    pub(crate) diagnostic: Diagnostic,
}
