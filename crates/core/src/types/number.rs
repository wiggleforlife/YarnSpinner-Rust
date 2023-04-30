//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner/Types/NumberType.cs>

use crate::function_wrappers::*;
use crate::prelude::types::TypeProperties;
use crate::prelude::*;
use std::ops::*;

/// A type that bridges to [`f32`]
pub(crate) fn number_type_properties() -> TypeProperties {
    TypeProperties::from_name("Number").with_methods(yarn_fn_registry! {
        Operator::EqualTo => RustType::eq_by_value,
        Operator::NotEqualTo => RustType::ne_by_value,
        Operator::Add => <RustType as Add>::add,
        Operator::Subtract => <RustType as Sub>::sub,
        Operator::Multiply => <RustType as Mul>::mul,
        Operator::Divide => <RustType as Div>::div,
        Operator::Modulo => <RustType as Rem>::rem,
        Operator::UnarySubtract => <RustType as Neg>::neg,
        Operator::GreaterThan => RustType::gt_by_value,
        Operator::GreaterThanOrEqualTo => RustType::ge_by_value,
        Operator::LessThan => RustType::lt_by_value,
        Operator::LessThanOrEqualTo => RustType::le_by_value,
    })
}

type RustType = f32;
