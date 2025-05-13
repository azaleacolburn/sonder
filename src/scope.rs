use crate::{data_model::VarData, lexer::CType};
use std::collections::HashMap;

/// Represents a single scope
/// While Rust functions can be nested inside functions
/// (allowing for currying of non-static variables)
/// C functions cannot, so this isn't represented
pub struct ScopeContext {
    pub scope_type: ScopeType,
    pub variables: HashMap<String, VarData>,
}

pub enum ScopeType {
    Function {
        ret: CType,
        args: Vec<(String, VarData)>,
    },
    Loop,
    Misc,
}
