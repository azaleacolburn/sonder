use std::{cell::RefCell, ops::Range, rc::Rc};

use crate::{analysis_ctx::AnalysisContext, lexer::CType};

pub type LineNumber = usize;

#[derive(Debug, Clone)]
pub struct VarData {
    // Usage / Block
    pub pointed_to: Vec<Rc<RefCell<Reference>>>, // References held to this variable
    pub usages: Vec<Usage>,

    // General Information
    pub var_type: CType,
    pub points_to: Vec<Rc<RefCell<Reference>>>, // References held by this variable
    pub rc: bool,
    pub clone: bool, // Cloned value (generated by sonder)
    pub is_mut: bool,

    // Struct handling
    pub instanceof_struct: Option<String>,
    pub fieldof_struct: Option<FieldInfo>,
}

impl VarData {
    pub fn new(
        var_type: CType,
        clone: bool,
        instanceof_struct: Option<String>,
        fieldof_struct: Option<FieldInfo>,
    ) -> Self {
        VarData {
            usages: Vec::new(),
            var_type,
            points_to: Vec::new(),
            pointed_to: Vec::new(),
            rc: false,
            clone,
            is_mut: false,
            instanceof_struct,
            fieldof_struct,
        }
    }

    /// Returns the outstanding reference held by this variable
    /// TODO Check if the last reference in the list is always the current reference
    pub fn current_reference_held(&self) -> Option<Rc<RefCell<Reference>>> {
        match self.points_to.last() {
            // Some(reference) if reference.borrow().start > line || reference.borrow().end > line => None,
            Some(reference) => Some(reference.clone()),
            None => None,
        }
    }

    pub fn reference_at_line(&self, line: LineNumber) -> Option<Rc<RefCell<Reference>>> {
        match self
            .points_to
            .iter()
            .find(|t| t.borrow().within_current_range(line))
        {
            Some(reference) => Some(reference.clone()),
            None => None,
        }
    }

    pub fn reference_to_var(&self, var_id: &str) -> Option<&Rc<RefCell<Reference>>> {
        self.points_to
            .iter()
            .find(|reference| reference.borrow().ref_to == var_id)
    }

    pub fn new_usage(&mut self, line: LineNumber) {
        // TODO Figure out how we're going to handle referring back to usages
        let usage = Usage::new(line, UsageType::RValue);
        self.usages.push(usage);

        if let Some(reference) = self.current_reference_held() {
            reference.borrow_mut().end = line;
        }
    }

    pub fn is_ptr(&self) -> bool {
        self.points_to.len() > 0
    }
}

/// Represents a singlular usage of a variable, not including its reference being taken

#[derive(Debug, Clone)]
pub struct Usage {
    line: LineNumber,
    usage_type: UsageType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UsageType {
    FunctionArg,
    RValue,
    LValue,
}

impl Usage {
    pub fn new(line: LineNumber, usage_type: UsageType) -> Self {
        Usage { line, usage_type }
    }

    pub fn get_line_number(&self) -> LineNumber {
        self.line
    }

    pub fn get_usage_type(&self) -> &UsageType {
        &self.usage_type
    }
}

/// Represents a span where a variable is behind a reference
/// A Reference is extended when the variable holding the reference is used
/// NOTe: Could be extended to Block soon

#[derive(Debug, Clone)]
pub struct Reference {
    reference_type: ReferenceType,
    ref_to: String,
    borrower: String,
    start: LineNumber,
    end: LineNumber,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    MutBorrowed,
    ConstBorrowed,
    MutPtr,
    ConstPtr,

    RcRefClone,
}

impl Reference {
    pub fn new(ref_to: impl ToString, borrower: impl ToString, line: LineNumber) -> Self {
        Reference {
            reference_type: ReferenceType::ConstBorrowed,
            ref_to: ref_to.to_string(),
            borrower: borrower.to_string(),
            start: line,
            end: line,
        }
    }

    pub fn construct_reference_chain(
        &self,
        ctx: &AnalysisContext,
        line: LineNumber,
    ) -> Vec<Reference> {
        let mut points_to = Vec::with_capacity(6);
        points_to.push(self.clone());

        let mut sub_id = self.get_reference_to();
        let mut maybe_reference = ctx
            .get_var(sub_id)
            .reference_at_line(line)
            .or_else(|| ctx.get_var(sub_id).current_reference_held());

        while let Some(reference) = maybe_reference {
            points_to.push(reference.borrow().clone());

            let b = reference.borrow();
            sub_id = b.get_reference_to();

            maybe_reference = ctx.get_var(&sub_id).reference_at_line(line);
        }

        return points_to;
    }

    // Non-inclusive on either end
    pub fn within_current_range(&self, line: usize) -> bool {
        self.start <= line && self.end >= line
    }

    pub fn set_mut(&mut self) {
        self.reference_type = ReferenceType::MutBorrowed;
    }

    pub fn set_rc(&mut self) {
        self.reference_type = ReferenceType::RcRefClone;
    }

    pub fn get_reference_to(&self) -> &str {
        &self.ref_to
    }

    pub fn get_borrower(&self) -> &str {
        &self.borrower
    }

    pub fn get_reference_type(&self) -> ReferenceType {
        self.reference_type.clone()
    }

    pub fn get_range(&self) -> Range<LineNumber> {
        Range {
            start: self.start,
            end: self.end,
        }
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum PtrType {
//     Rc,
//     RcRefClone,
//     RefCell,
//     RawPtrMut,
//     RawPtrImut,
//     MutRef,
//     ImutRef,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct StructData {
    pub field_definitions: Vec<FieldDefinition>,
}

impl StructData {
    pub fn mut_field<F>(&mut self, field: String, f: F)
    where
        F: FnOnce(&mut FieldDefinition),
    {
        let field_data: &mut FieldDefinition = self
            .field_definitions
            .iter_mut()
            .find(|field_data| field_data.id == field)
            .unwrap();

        f(field_data)
    }
}

/// Collected during declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub struct_id: String,
    pub field_id: String,
}

/// Collected during definition
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub id: String,
    pub ptr_type: Vec<ReferenceType>,
    pub c_type: CType,
}
