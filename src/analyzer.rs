use std::{cmp, collections::HashMap, ops::Range};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtrData {
    pub points_to: String,
    pub mutates: bool,
    pub ptr_type: Vec<PtrType>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarData<'a> {
    pub ptr_data: Option<PtrData>,
    pub pointed_to_by: Vec<&'a str>,
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
    prev_vars: &HashMap<String, VarData<'a>>,
) -> HashMap<String, VarData<'a>> {
    let mut vars: HashMap<String, VarData> = prev_vars.clone();
    if root.children.is_some() {
        root.children.as_ref().unwrap().iter().for_each(|node| {
            determine_var_mutability(node, &vars)
                .iter()
                .for_each(|(id, var_data)| {
                    vars.insert(id.to_string(), var_data.clone());
                });
        })
    }

    match &root.token {
        NodeType::Declaration(id, _, _) => {
            println!("Declaration: {id}");
            vars.insert(
                id.to_string(),
                VarData {
                    ptr_data: None,
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
            vars.entry(id.to_string()).and_modify(|var_data| {
                var_data.is_mut_direct = true;
                var_data.add_non_borrowed_line(root.line);
            });
        }
        NodeType::PtrDeclaration(id, _, expr) => {
            determine_var_mutability(expr, &vars)
                .iter()
                .for_each(|(id, var_data)| {
                    vars.insert(id.to_string(), var_data.clone());
                });

            let expr_ids = find_ids(expr);
            if expr_ids.len() > 1 {
                panic!("More than one id in expr: `&(a + b)` not legal");
            } else if expr_ids.len() != 1 {
                panic!("ptr to no id");
            }
            // As we go, we replace certain elements in this vector with `PtrType::MutRef`
            let ptr_chain_placeholder_types =
                traverse_pointer_chain(&expr_ids[0], &vars, 0, u8::MAX)
                    .iter()
                    .map(|_| PtrType::ImutRef)
                    .collect();
            let ptr_data = Some(PtrData {
                points_to: expr_ids[0].clone(),
                mutates: false,
                ptr_type: ptr_chain_placeholder_types,
            });
            let var = VarData {
                ptr_data,
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
            vars.insert(id.to_string(), var);
            // Doesn't support &that + &this
            // This immediantly breakes borrow checking rules
            vars.entry(expr_ids[0].to_string()).and_modify(|var_data| {
                var_data.pointed_to_by.push(id);
                var_data.add_non_borrowed_line(root.line);
            });
        }
        NodeType::DerefAssignment(_, l_side) => {
            let deref_ids = find_ids(&l_side);
            // This breakes because `*(t + s) = bar` is not allowed
            // However, **m is fine
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            } else if deref_ids.len() != 1 {
                panic!("Unsupported: no_ids being dereffed")
            }
            let num_of_vars = count_derefs(&l_side) + 1;
            let mut ptr_chain = traverse_pointer_chain(&deref_ids[0], &vars, 0, num_of_vars)
                .into_iter()
                .rev();
            // eg. [m, p, n]
            let first_ptr = ptr_chain.next().expect("No pointers in chain");
            vars.entry(first_ptr.clone()).and_modify(|var_data| {
                var_data.add_non_borrowed_line(root.line);
                var_data
                    .ptr_data
                    .as_mut()
                    .expect("First ptr in deref not ptr")
                    .mutates = true;
                var_data
                    .ptr_data
                    .as_mut()
                    .expect("First ptr in deref not ptr")
                    .ptr_type
                    .fill(PtrType::MutRef);
            });

            ptr_chain.clone().enumerate().for_each(|(i, var)| {
                if i != ptr_chain.len() - 1 {
                    vars.entry(var.clone()).and_modify(|var_data| {
                        var_data
                            .ptr_data
                            .as_mut()
                            .expect("Non-last in chain not ptr")
                            .mutates = true;
                        let type_chain = &mut var_data
                            .ptr_data
                            .as_mut()
                            .expect("Non-last in chain not ptr")
                            .ptr_type;
                        for type_chain_i in i..type_chain.len() {
                            type_chain[type_chain_i] = PtrType::MutRef;
                        }
                    });
                }
                vars.entry(var).and_modify(|var_data| {
                    var_data.is_mut_by_ptr = true;
                });
            });
        }
        NodeType::Id(id) => {
            vars.entry(id.to_string())
                .and_modify(|var_data| var_data.add_non_borrowed_line(root.line));
        }
        NodeType::Adr(id) => {
            vars.entry(id.to_string())
                .and_modify(|var_data| var_data.new_borrow(root.line));
        }
        _ => {}
    };
    vars
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

fn traverse_pointer_chain<'a>(
    root: &'a str,
    var_info: &HashMap<String, VarData<'a>>,
    total_depth: u8,
    max_depth: u8,
) -> Vec<String> {
    if total_depth == max_depth {
        return vec![];
    }
    let is_ptr = &var_info
        .get(root)
        .as_ref()
        .expect("Root not found in map")
        .ptr_data;
    match is_ptr {
        Some(ref ptr_data) => {
            let mut vec =
                traverse_pointer_chain(&ptr_data.points_to, var_info, total_depth + 1, max_depth);
            vec.push(root.to_string());
            vec
        }
        None => {
            vec![root.to_string()]
        }
    }
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
