use crate::{
    analyzer::StructData,
    data_model::{LineNumber, Reference, StructData, VarData},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// The top-level datastructure that stores data about all the variables and referencing
/// Stores a vector of the instances of addresses being taken, in order
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub variables: HashMap<String, VarData>,
    pub structs: HashMap<String, StructData>,
}

impl AnalysisContext {
    pub fn new() -> AnalysisContext {
        AnalysisContext {
            variables: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    pub fn declaration(&mut self, id: impl ToString, data: VarData) {
        self.variables.insert(id.to_string(), data);
    }

    pub fn new_usage(&mut self, id: &str, line: LineNumber) {
        let initial_var = self.variables.get_mut(id).expect("Var not in ctx");
        initial_var.new_usage(line);
    }

    pub fn assignment(&mut self, assigned_to: &str) {
        let l_value = self.variables.get_mut(assigned_to).expect("Var not in ctx");
        l_value.is_mut = true;
    }

    pub fn ptr_assignment(&mut self, borrowed: &str, assigned_to: &str, line: LineNumber) {
        self.assignment(assigned_to);

        let new_reference = Rc::new(RefCell::new(Reference::new(borrowed, assigned_to, line)));

        let l_value = self.variables.get_mut(assigned_to).expect("Var not in ctx");
        l_value.references.push(new_reference)
    }

    // TODO Figure out how to recursively mark things as mutable
    pub fn deref_assignment(&mut self, assigned_to: &str, line: LineNumber) {
        let l_value = self.variables.get_mut(assigned_to).expect("Var not in ctx");
        assert!(l_value.is_ptr());
        l_value.new_usage(line);

        let reference_data = l_value.current_reference_held().expect("Null ptr deref");
        self.assignment(reference_data.borrow().get_reference_to());
    }

    pub fn struct_declaration(&mut self, id: String, struct_data: StructData) {
        self.structs.insert(id, struct_data);
    }

    pub fn new_struct(&mut self, id: impl ToString, data: StructData) {
        self.structs.insert(id.to_string(), data);
    }

    pub fn get_struct(&self, id: &str) -> &StructData {
        self.structs.get(id).expect("Struct not in map")
    }

    pub fn get_var(&self, id: &str) -> &VarData {
        println!("var_id: {id}");
        self.variables.get(id).expect("Var not in map")
    }

    pub fn mut_var<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(&mut VarData),
    {
        self.variables.entry(id).and_modify(f);
    }

    pub fn mut_struct<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(&mut StructData),
    {
        self.structs.entry(id).and_modify(f);
    }

    pub fn construct_ptr_chain(&self, root: String, total_depth: u8, max_depth: u8) -> Vec<String> {
        if total_depth == max_depth {
            return vec![];
        }
        let ptr_data = &self
            .variables
            .get(&root)
            .as_ref()
            .expect("Root in construct ptr chain not found in map")
            .ptr_to;

        match ptr_data.is_empty() {
            false => {
                let mut chain = self.construct_ptr_chain(
                    ptr_data
                        .last()
                        .unwrap()
                        .borrow()
                        .get_reference_to()
                        .to_string(),
                    total_depth + 1,
                    max_depth,
                );
                chain.push(root.to_string());
                chain
            }
            true => vec![root.to_string()],
        }
    }
}
