use crate::data_model::{LineNumber, Reference, VarData};
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
}

impl AnalysisContext {
    pub fn new() -> AnalysisContext {
        AnalysisContext {
            variables: HashMap::new(),
        }
    }
    pub fn new_var(&mut self, id: String, data: VarData) {
        self.variables.insert(id, data);

        // data.ptr_to.iter().for_each(|reference| reference)
    }

    pub fn new_usage(&mut self, id: &str, line: LineNumber) {
        let initial_var = self.variables.get_mut(id).expect("Var not in ctx");
        initial_var.new_usage(line);
    }

    pub fn assignment(&mut self, id: &str) {
        let l_value = self.variables.get_mut(id).expect("Var not in ctx");
        l_value.is_mut = true;
        if l_value.ptr_to
        l_value.ptr
    }

    pub fn new_adr(&mut self, adr: AdrData, var: Option<String>) {
        self.addresses.push(adr.clone());

        if let Some(var) = var {
            self.variables.entry(var).and_modify(|var_data| {
                var_data.addresses.push(adr.clone());
            });
        }
    }

    pub fn new_struct(&mut self, id: String, struct_data: StructData) {
        self.structs.insert(id, struct_data);
    }

    pub fn get_struct(&self, id: &str) -> &StructData {
        self.structs.get(id).expect("Struct not in map")
    }

    pub fn get_var(&self, id: &str) -> &VarData {
        println!("var_id: {id}");
        self.variables.get(id).expect("Var not in map")
    }

    /// Gets an address, given the id the address points to
    /// If more than one exists, the first one is returned
    pub fn get_adr(&self, var_id: &str) -> &Rc<RefCell<AdrData>> {
        self.addresses
            .iter()
            .find(|adr_data| adr_data.borrow().adr_of == var_id)
            .expect("Address not in map")
    }

    pub fn mut_var<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(&mut VarData),
    {
        self.variables.entry(id).and_modify(f);
    }

    /// Applies a function to an adr_data given the underlying id the adr points to
    pub fn mut_adr<F>(&mut self, id: &str, f: F)
    where
        F: FnOnce(RefMut<AdrData>),
    {
        let adr_data = self
            .addresses
            .iter_mut()
            .map(|adr_cell| adr_cell.clone())
            .find(|adr_data| adr_data.borrow().adr_of == id)
            .expect(format!("No adr that points to given id: {id}").as_str());

        f(RefCell::borrow_mut(&adr_data))
    }

    pub fn mut_struct<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(&mut StructData),
    {
        self.structs.entry(id).and_modify(f);
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
                let mut chain = self.traverse_pointer_chain(
                    ptr_data.last().unwrap().borrow().adr_of.clone(),
                    total_depth + 1,
                    max_depth,
                );
                chain.push(root.to_string());
                chain
            }
            true => vec![root.to_string()],
        }
    }
    /// Finds which reference a specific variable held at the given line number
    /// Panics if:
    /// - Variable was declared on line given <- up for question
    /// - Variable doesn't exist in variables
    /// - Variable doesn't exist on given line
    /// - Variable not a ptr or never initialized
    pub fn find_which_ref_at_id(&self, var_id: &str, line: usize) -> String {
        let init_at = self
            .variables
            .get(var_id)
            .expect("Variable given doesn't exist")
            .non_borrowed_lines[0]
            .start;
        // TODO: Check if this should be > or >=
        println!("var_id: {var_id} init_at: {init_at} line: {line}");
        assert!(init_at < line);

        self.variables
            .get(var_id)
            .expect("Variable given doesn't exist")
            .addresses
            .iter()
            .map(|adr_data| adr_data.borrow())
            .filter(|adr_data_ref| adr_data_ref.line_taken < line)
            .fold(String::new(), |_, adr_data_ref| adr_data_ref.adr_of.clone())
    }

    pub fn print_refs(&self) {
        self.variables.iter().for_each(|(id, var_data)| {
            println!("{id}:");
            var_data
                .non_borrowed_lines
                .iter()
                .for_each(|non_borrowed_range| println!("\t{:?}", non_borrowed_range))
        })
    }
}
