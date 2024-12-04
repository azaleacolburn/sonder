use std::{cell::RefCell, cmp, collections::HashMap, ops::Range, rc::Rc};

use crate::parser::{NodeType, TokenNode as Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtrType {
    Rc,
    RcClone,
    RefCell,
    RawPtrMut,
    RawPtrImut,
    MutRef,
    ImutRef,
}

/// The top-level datastructure that stores data about all the variables and referencing
/// Stores a vector of the instances of addresses being taken, in order
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisContext<'a> {
    pub variables: HashMap<String, VarData<'a>>,
    pub addresses: Vec<Rc<RefCell<AdrData<'a>>>>,
}

impl<'a> AnalysisContext<'a> {
    pub fn new() -> AnalysisContext<'a> {
        AnalysisContext {
            variables: HashMap::new(),
            addresses: vec![],
        }
    }
    pub fn new_var(&mut self, id: String, data: VarData<'a>) {
        self.variables.insert(id, data);
    }
    pub fn new_adr(&mut self, adr: Rc<RefCell<AdrData<'a>>>, var: Option<String>) {
        self.addresses.push(adr.clone());

        if let Some(var) = var {
            self.variables.entry(var).and_modify(|var_data| {
                var_data.addresses.push(adr.clone());
            });
        }
    }

    pub fn get_var(&self, id: &str) -> Option<&VarData<'a>> {
        self.variables.get(id)
    }

    /// Gets an address, given the id the address points to
    pub fn get_adr(&self, var_id: &str) -> Option<&Rc<RefCell<AdrData<'a>>>> {
        self.addresses
            .iter()
            .find(|adr_data| adr_data.borrow().adr_of == var_id)
    }

    pub fn mut_var<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(&mut VarData),
    {
        self.variables.entry(id).and_modify(f);
    }

    pub fn is_ptr(&self, id: &String) -> bool {
        self.variables
            .get(id)
            .as_ref()
            .expect("Checked ptr not in ctx")
            .addresses
            .len()
            > 0
    }
    pub fn traverse_pointer_chain(
        &self,
        root: String,
        total_depth: u8,
        max_depth: u8,
    ) -> Vec<String> {
        if total_depth == max_depth {
            return vec![];
        }
        let ptr_data = &self
            .variables
            .get(&root)
            .as_ref()
            .expect("Root in traversing ptr chain not found in map")
            .addresses;

        match ptr_data.is_empty() {
            false => {
                let mut vec = self.traverse_pointer_chain(
                    ptr_data.last().unwrap().borrow().adr_of.clone(),
                    total_depth + 1,
                    max_depth,
                );
                vec.push(root.to_string());
                vec
            }
            true => {
                vec![root.to_string()]
            }
        }
    }
    /// Finds which reference a specific variable held at the given line number
    /// Panics if:
    /// - Variable was declared on line given <- up for question
    /// - Variable doesn't exist in variables
    /// - Variable doesn't exist on given line
    /// - Variable not a ptr or never initialized
    pub fn find_which_ref_at_id(&self, var_id: &str, line: usize) -> String {
        let mut reference: Option<String> = None;
        let init_at = self
            .variables
            .get(var_id)
            .expect("Variable given doesn't exist")
            .non_borrowed_lines[0]
            .start;
        // TODO: Check if this should be > or >=
        assert!(init_at >= line);
        self.variables
            .get(var_id)
            .expect("Variable given doesn't exist")
            .addresses
            .iter()
            .for_each(|adr_data| {
                let adr_data_ref = adr_data.borrow();
                if adr_data_ref.line_taken < line {
                    reference = Some(adr_data_ref.adr_of.clone());
                }
            });
        if reference.is_none() {
            panic!("Reference was none in finding ref function");
        }
        reference.unwrap()
    }
}
/// Data of a specific instance of the address of a variable being taken
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrData<'a> {
    pub adr_of: String,
    pub mutates: bool,
    pub held_by: Option<&'a str>,
    // Determine if ptrtype makes sense in a variable-independent context
    pub ptr_type: Vec<PtrType>,
    pub line_taken: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarData<'a> {
    // An emptry vector indicates a non-ptr variable
    pub addresses: Vec<Rc<RefCell<AdrData<'a>>>>,
    // The type of ptr here is relevent for annotating adr
    // Reference order matters here, so we have to be careful
    pub pointed_to_by: Vec<String>,
    pub is_mut_by_ptr: bool,
    pub is_mut_direct: bool,
    pub rc: bool,
    set_start_borrow: bool, // do we need to set the start of the new borrow
    // The pattern of initializing and instantiating seperately is harder to analyze and requires a PtrAssignment node
    // Line ranges when the var isn't borrowed
    pub non_borrowed_lines: Vec<Range<usize>>,
}

impl<'a> VarData<'a> {
    pub fn add_non_borrowed_line(&mut self, line: usize) {
        let len = self.non_borrowed_lines.len() - 1;
        let end = &mut self.non_borrowed_lines[len].end;
        *end = cmp::max(line, *end);
        if self.set_start_borrow {
            let start = &mut self.non_borrowed_lines[len].start;
            *start = cmp::max(line, *start);
            self.set_start_borrow = false;
        }
    }
    pub fn new_borrow(&mut self, line: usize) {
        self.non_borrowed_lines.push(Range {
            start: line,
            end: line,
        });
        self.set_start_borrow = true;
    }
}

pub fn determine_var_mutability<'a>(
    root: &'a Node,
    prev_ctx: AnalysisContext<'a>,
) -> AnalysisContext<'a> {
    let mut ctx: AnalysisContext = prev_ctx;
    if root.children.is_some() {
        root.children.as_ref().unwrap().iter().for_each(|node| {
            // TODO: This feels illegal
            ctx = determine_var_mutability(node, ctx);
        })
    }

    match &root.token {
        NodeType::Declaration(id, _, _) => {
            println!("Declaration: {id}");
            ctx.new_var(
                id.to_string(),
                VarData {
                    addresses: vec![],
                    pointed_to_by: vec![],
                    is_mut_by_ptr: false,
                    is_mut_direct: false,
                    rc: false,
                    non_borrowed_lines: vec![Range {
                        start: root.line,
                        end: root.line,
                    }],
                    set_start_borrow: false,
                },
            );
        }
        NodeType::Assignment(_, id) => {
            ctx.mut_var(id.to_string(), |var_data| {
                var_data.is_mut_direct = true;
                var_data.add_non_borrowed_line(root.line);
            });
        }
        NodeType::PtrDeclaration(id, _, expr) => {
            ctx = determine_var_mutability(expr, ctx);

            let ids = find_ids(&expr);
            let expr_ptrs: Vec<&String> = ids.iter().filter(|id| ctx.is_ptr(id)).collect();

            let mut ptr_type_chain: Vec<PtrType> = if expr_ptrs.len() == 1 {
                // As we go, we replace certain elements in this vector with `PtrType::MutRef`
                ctx.traverse_pointer_chain(expr_ptrs[0].clone(), 0, u8::MAX)
                    .iter()
                    .map(|_| PtrType::ImutRef)
                    .collect()
            } else if expr_ptrs.len() > 1 {
                // Ptr arithmatic outside the context of arrays is automatically a raw ptr
                vec![PtrType::RawPtrImut]
            } else {
                vec![PtrType::ImutRef]
            };

            let addresses = find_addresses(&expr);
            // TODO: Add ptr chain len for non-overlapping ptrs
            let points_to = if addresses.len() == 1 {
                addresses.last()
            } else if addresses.len() > 1 {
                if let Some(last) = ptr_type_chain.last_mut() {
                    *last = PtrType::RawPtrImut;
                }
                addresses.last()
            } else if addresses.len() == 0 {
                if expr_ptrs.len() == 0 {
                    // TODO: Allow empty expressions in the future
                    panic!("PtrDeclaration to nothing");
                }
                expr_ptrs.last().map(|n| &**n)
            } else {
                panic!(".len() is a usize");
            }
            .unwrap();

            let adr_data = ctx
                .get_adr(&points_to)
                .expect("Points to algorithm failed")
                .clone();
            adr_data.borrow_mut().held_by = Some(&id);

            let var = VarData {
                addresses: vec![adr_data],
                pointed_to_by: vec![],
                is_mut_by_ptr: false,
                is_mut_direct: false,
                rc: false,
                non_borrowed_lines: vec![Range {
                    start: root.line,
                    end: root.line,
                }],
                set_start_borrow: false,
            };
            // TODO: Figure out how to annotate specific address call as mutable or immutable
            ctx.new_var(id.to_string(), var);
            // Doesn't support &that + &this
            // This immediantly breakes borrow checking rules
            ctx.mut_var(points_to.to_string(), |var_data| {
                var_data.pointed_to_by.push(id.clone());
            });
        }
        NodeType::DerefAssignment(_, l_side) => {
            ctx = determine_var_mutability(&l_side, ctx);
            let deref_ids = find_ids(&l_side);
            // This breakes because `*(t + s) = bar` is not allowed
            // However, **m is fine
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            } else if deref_ids.len() != 1 {
                panic!("Unsupported: no_ids being dereffed")
            }
            let num_of_vars = count_derefs(&l_side) + 1;
            let mut ptr_chain = ctx
                .traverse_pointer_chain(deref_ids[0].clone(), 0, num_of_vars)
                .into_iter()
                .rev();
            // eg. [m, p, n]
            let first_ptr = ptr_chain.next().expect("No pointers in chain");
            ctx.mut_var(first_ptr.clone(), |var_data| {
                var_data.add_non_borrowed_line(root.line);
                let adr = var_data.addresses.last().expect("Variable not ptr");
                adr.borrow_mut().mutates = true;
                adr.borrow_mut().ptr_type.fill(PtrType::MutRef);
            });

            ptr_chain.clone().enumerate().for_each(|(i, var)| {
                if i != ptr_chain.len() - 1 {
                    ctx.mut_var(var.clone(), |var_data| {
                        let mut adr = var_data
                            .addresses
                            .last()
                            .expect("Variable not ptr")
                            .borrow_mut();

                        adr.mutates = true;

                        for type_chain_i in i..adr.ptr_type.len() {
                            adr.ptr_type[type_chain_i] = PtrType::MutRef;
                        }
                    });
                }
                ctx.mut_var(var, |var_data| var_data.is_mut_by_ptr = true);
            });
        }
        NodeType::Id(id) => {
            ctx.mut_var(id.to_string(), |var_data| {
                var_data.add_non_borrowed_line(root.line)
            });
        }
        NodeType::Adr(id) => {
            let ptr_type_chain = ctx
                .traverse_pointer_chain(id.clone(), 0, u8::MAX)
                .iter()
                .map(|_| PtrType::ImutRef)
                .collect();
            let adr_data = Rc::new(RefCell::new(AdrData {
                adr_of: id.to_string(),
                mutates: false,
                held_by: None,
                ptr_type: ptr_type_chain,
                line_taken: root.line,
            }));
            // We don't know if a variable owns this ref yet
            // that's for the ptr_declaration to figure out
            ctx.new_adr(adr_data, None);
            ctx.mut_var(id.to_string(), |var_data| var_data.new_borrow(root.line));
        }
        NodeType::DeRef(adr) => {
            let ids = find_ids(&adr);
            // Panics if more than one id derefed
            if ids.len() != 1 {
                panic!("more than one or 0 ids derefed");
            }
            let id = ids[0].clone();
            ctx.mut_var(id, |var_data| var_data.add_non_borrowed_line(root.line));
        }
        _ => {}
    };
    ctx
}

pub fn find_addresses(root: &Node) -> Vec<String> {
    let mut vec = match root.children.as_ref() {
        Some(children) => children
            .iter()
            .flat_map(find_addresses)
            .collect::<Vec<String>>(),
        None => vec![],
    };
    match &root.token {
        NodeType::Adr(id) => vec.push(id.to_string()),
        _ => {}
    }
    vec
}

pub fn count_derefs(root: &Node) -> u8 {
    let mut count = 0;
    let children = root.children.as_ref();
    if let Some(children) = children {
        count += children.iter().map(count_derefs).sum::<u8>();
    }
    match &root.token {
        NodeType::DeRef(expr) => {
            count += count_derefs(&expr) + 1;
        }
        _ => {}
    };
    count
}

pub fn find_ids<'a>(root: &'a Node) -> Vec<String> {
    let mut ids: Vec<String> = root
        .children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(find_ids)
        .collect();
    match &root.token {
        NodeType::Id(id) => ids.push(id.to_string()),
        NodeType::Adr(id) => ids.push(id.to_string()),
        NodeType::DeRef(node) => {
            ids.append(&mut find_ids(&*node));
        }
        _ => {}
    }

    ids
}
