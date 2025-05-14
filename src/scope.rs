use crate::{data_model::VarData, lexer::CType};
use std::collections::HashMap;

/// Represents a single scope
/// While Rust functions can be nested inside functions
/// (allowing for currying of non-static variables)
/// C functions cannot, so this isn't represented
#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub scope_type: ScopeType,
    pub variables: HashMap<String, VarData>,
}

impl ScopeContext {
    pub fn new(scope_type: ScopeType) -> ScopeContext {
        ScopeContext {
            scope_type,
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Function {
        name: String,
        ret: CType,
        args: Vec<String>,
    },
    Loop,
    Top,
    Misc,
}
