use crate::{
    analysis_ctx::AnalysisContext,
    analyzer::{count_derefs, find_ids},
    ast::{AssignmentOpType, NodeType, TokenNode as Node},
    data_model::{FieldDefinition, Reference, ReferenceType},
    lexer::CType,
};
use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct AnnotatedNode {
    pub token: AnnotatedNodeT,
    pub children: Vec<AnnotatedNode>,
}

#[derive(Debug, Clone)]
pub enum AnnotatedNodeT {
    Program {
        imports: Vec<String>,
    },
    Sub,
    Div,
    Eq,
    Id {
        id: String,
        rc: bool,
    }, // figure out if we want this here
    EqCmp,
    NeqCmp,
    BOr,
    BAnd,
    BXor,
    BOrEq,
    BAndEq,
    BXorEq,
    SubEq,
    AddEq,
    DivEq,
    MulEq,
    Mul,
    _MNeg,
    AndCmp,
    OrCmp,
    NumLiteral(usize),
    Add,
    If,
    For,
    While,
    _Loop,
    Break,
    FunctionCall(String),
    Scope(Option<CType>), // <-- anything that has {} is a scope, scope is how we're handling multiple statements, scopes return the last statement's result or void
    Assignment {
        op: AssignmentOpType,
        id: String,
        rc: bool,
    },
    DerefAssignment {
        op: AssignmentOpType,
        id: String,
        rc: bool,
        // This is the type of each reference being dereferenced, not in total
        ref_types: Vec<ReferenceType>,
    },
    Declaration {
        id: String,
        is_mut: bool,
        t: CType,
        rc: bool,
        is_used: bool,
        init_value_unused: bool,
    },
    PtrDeclaration {
        id: String,
        is_mut: bool,
        points_to: Vec<Rc<RefCell<Reference>>>,
        t: CType,
        adr: Box<AnnotatedNode>,
        ref_type: Vec<ReferenceType>,
        // Refers to it being an rc_ptr itself, not a
        rc: bool,
        is_used: bool,
        init_value_unused: bool,
    },
    Asm(String),
    // This is handled by the ptr declaration for now
    Adr {
        id: String,
    },
    DeRef {
        id: String,
        rc: bool,
        count: u8,
    },
    ArrayDeclaration {
        id: String,
        t: CType,
        size: usize,
        is_used: bool,
        is_mut: bool,
        items: Vec<AnnotatedNode>,
        init_value_unused: bool,
    },
    FunctionDeclaration {
        id: String,
        t: CType,
    },
    Assert,
    Return {
        expr: Box<AnnotatedNode>,
    },
    PutChar,
    StructDefinition {
        struct_id: String,
        field_definitions: Vec<FieldDefinition>,
        has_ref: bool,
    },
    StructDeclaration {
        var_id: String,
        struct_id: String,
        is_mut: bool,
        fields: Vec<(FieldDefinition, AnnotatedNode)>,
        is_used: bool,
        init_value_unused: bool,
    },
    StructFieldAssignment {
        var_id: String,
        field_id: String,
        op: AssignmentOpType,
        expr: Box<AnnotatedNode>,
    },
}

impl Display for AnnotatedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token) // doesn't print values
    }
}

impl AnnotatedNode {
    pub fn print(&self, n: &mut i32) {
        (0..*n).into_iter().for_each(|_| print!("\t"));
        println!("{}", self);
        *n += 1;
        self.children.iter().for_each(|node| {
            node.print(n);
        });
        *n -= 1;
    }
}

impl Node {
    pub fn annotate<'a>(&self, ctx: &AnalysisContext) -> AnnotatedNode {
        let root = self;

        let token = match &root.token {
            NodeType::Declaration(id, t, _) => {
                let declaration_info = ctx.get_var(id);
                let is_used = declaration_info.usages.len() > 0;
                println!("{id}: {is_used}");
                let init_value_unused = declaration_info.init_value_unused;

                AnnotatedNodeT::Declaration {
                    id: id.to_string(),
                    is_mut: declaration_info.is_mut,
                    t: t.clone(),
                    rc: declaration_info.rc,
                    is_used,
                    init_value_unused,
                }
            }
            NodeType::PtrDeclaration(id, t, adr) => {
                let ptr_var_info = ctx.get_var(id);
                let annotated_adr = Box::new(adr.annotate(ctx));

                let points_to = ptr_var_info.points_to.clone();

                let reference = points_to[0].clone();

                let ref_type: Vec<ReferenceType> = reference
                    .borrow()
                    .construct_reference_chain(ctx, root.line)
                    .iter()
                    .map(Reference::get_reference_type)
                    .collect();

                let is_used = ptr_var_info.usages.len() > 0;
                let init_value_unused = ptr_var_info.init_value_unused;

                AnnotatedNodeT::PtrDeclaration {
                    id: id.to_string(),
                    is_mut: ptr_var_info.is_mut,
                    points_to,
                    t: t.clone(),
                    ref_type,
                    adr: annotated_adr,
                    rc: ptr_var_info.rc,
                    is_used,
                    init_value_unused,
                }
            }
            NodeType::Adr(id) => {
                // `(&mut t + (&b))` illegal
                // `&mut &mut &t` illegal
                // Unsafe assumption: Adresses are always immutable unless explicitely annotated otherwise by the ptr declaration
                // `list.append(&mut other_list)` isn't something we're going to worry about for now
                AnnotatedNodeT::Adr { id: id.to_string() }
            }
            // It seems like assignments and deref assignments need to handle referencing themselves
            // Unless we want Adr nodes to know what kind of reference they are (which actually is
            // sounding like the right decision now)
            NodeType::DerefAssignment(op, adr) => {
                let count = count_derefs(adr); // TODO Maybe fix function

                let derefed_id = find_ids(&adr)[0].clone();
                let ptr_data = ctx.get_var(&derefed_id);

                let reference = ptr_data
                    .reference_at_line(root.line)
                    .expect("Non-ptr derefed on lside");

                let mut ref_types: Vec<ReferenceType> = reference
                    .borrow()
                    .construct_reference_chain(ctx, root.line)
                    .iter()
                    .map(Reference::get_reference_type)
                    .collect();

                ref_types.truncate(count as usize);

                let rc = ctx.get_var(&derefed_id).rc;
                AnnotatedNodeT::DerefAssignment {
                    op: op.clone(),
                    id: derefed_id.clone(),
                    rc,
                    ref_types,
                }
            }
            NodeType::DeRef(expr) => {
                let count = count_derefs(expr) + 1;

                let ids = find_ids(&expr);
                let derefed_id = ids[0].clone();

                let var_data = ctx.get_var(&derefed_id);
                let reference = var_data
                    .reference_at_line(root.line)
                    .expect("derefed id not ptr");

                let b = reference.borrow();
                let sub_id = b.get_reference_to();

                let rc = ctx.get_var(&sub_id).rc;
                AnnotatedNodeT::DeRef {
                    id: derefed_id.clone(),
                    rc,
                    count,
                }
            }
            NodeType::Id(id) => {
                let rc = ctx.get_var(id).rc;
                AnnotatedNodeT::Id {
                    id: id.to_string(),
                    rc,
                }
            }
            NodeType::Program => {
                // TODO: Check if some "count as we go" solution might work better
                let rc = false;
                let refcell = false;
                let mut rcclone = false;
                ctx.current_scope().variables.iter().for_each(|(_, data)| {
                    data.points_to.iter().for_each(|reference_block| {
                        match reference_block.as_ref().borrow().get_reference_type() {
                            ReferenceType::RcRefClone => rcclone = true,
                            // PtrType::RefCell => refcell = true,
                            // PtrType::RcRefClone => rcclone = true,
                            _ => {}
                        }
                    })
                });
                let mut imports: Vec<String> = vec![];
                if rc {
                    imports.push(String::from("use std::rc::Rc;"))
                }
                if refcell {
                    imports.push(String::from("use std::cell::RefCell;"))
                }
                if rcclone {
                    imports.push(String::from("use std::{cell::RefCell, rc::Rc};"))
                }

                AnnotatedNodeT::Program { imports }
            }
            NodeType::Assignment(op, id) => {
                let rc = ctx.get_var(id).rc;
                AnnotatedNodeT::Assignment {
                    id: id.clone(),
                    op: op.clone(),
                    rc,
                }
            }
            NodeType::StructDefinition {
                struct_id,
                field_definitions: _, // Field Definitions gathered by the parser
            } => {
                // Field definitions gathered by the analyzer (smart ptr type chain)
                let analyzed_field_definitions =
                    ctx.get_struct(struct_id).field_definitions.clone();
                let has_ref = analyzed_field_definitions
                    .iter()
                    .any(|field| field.ptr_type.len() > 0);

                AnnotatedNodeT::StructDefinition {
                    struct_id: struct_id.clone(),
                    field_definitions: analyzed_field_definitions,
                    has_ref,
                }
            }
            NodeType::StructDeclaration {
                var_id,
                struct_id,
                exprs,
            } => {
                let var_data = ctx.get_var(var_id);
                let field_definitions = ctx.get_struct(&struct_id).field_definitions.clone();
                // TODO: Annotate node properly for ptrs
                // NOTE: Will panic is invalid compound literal
                // TODO: Add checks for compound literal
                let fields: Vec<(FieldDefinition, AnnotatedNode)> = exprs
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, node)| (field_definitions[i].clone(), node.annotate(ctx)))
                    .collect();

                let is_used = var_data.usages.len() > 0;
                let init_value_unused = var_data.init_value_unused;

                AnnotatedNodeT::StructDeclaration {
                    var_id: var_id.clone(),
                    struct_id: struct_id.clone(),
                    is_mut: var_data.is_mut,
                    fields,
                    is_used,
                    init_value_unused,
                }
            }
            NodeType::ArrayDeclaration(id, c_type, count) => {
                let var = ctx.get_var(id);
                let is_mut = var.is_mut;
                let is_used = var.usages.len() != 0;
                let items: Vec<AnnotatedNode> = match root.children.as_ref() {
                    Some(vec) => vec.iter().map(|node| node.annotate(ctx)).collect(),
                    None => Vec::new(),
                };
                let init_value_unused = var.init_value_unused;

                AnnotatedNodeT::ArrayDeclaration {
                    id: id.clone(),
                    t: c_type.clone(),
                    size: *count,
                    is_used,
                    is_mut,
                    items,
                    init_value_unused,
                }
            }
            NodeType::StructFieldAssignment {
                var_id,
                field_id,
                assignment_op,
                expr,
            } => AnnotatedNodeT::StructFieldAssignment {
                var_id: var_id.clone(),
                field_id: field_id.clone(),
                op: assignment_op.clone(),
                expr: Box::new(expr.annotate(ctx)),
            },
            NodeType::Return { expr } => AnnotatedNodeT::Return {
                expr: Box::new(expr.annotate(ctx)),
            },
            node => node.to_annotated_node(),
        };
        let children = root.children.as_ref();
        let annotated_node_children = match children {
            Some(children) => children.iter().map(|node| node.annotate(ctx)).collect(),
            None => Vec::new(),
        };

        AnnotatedNode {
            token,
            children: annotated_node_children,
        }
    }
}
