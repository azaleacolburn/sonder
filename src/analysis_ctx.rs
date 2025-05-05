use crate::{
    data_model::{LineNumber, Reference, ReferenceType, StructData, UsageType, VarData},
    lexer::CType,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

    pub fn new_usage(&mut self, id: &str, line: LineNumber, t: UsageType) {
        let initial_var = self.variables.get_mut(id).expect("Var not in ctx");
        initial_var.new_usage(line, t);
    }

    pub fn assignment(&mut self, assigned_to: &str, rvalue_ids: Vec<String>, line: LineNumber) {
        rvalue_ids.iter().for_each(|id| {
            let var_data = self.get_var_mut(id);
            var_data.new_usage(line, UsageType::RValue);
        });

        let l_value_data = self.get_var(assigned_to);
        if l_value_data.fieldof_struct.is_some() {
            let struct_var_id = assigned_to.split(".").nth(0).unwrap().to_string();
            self.mut_var(struct_var_id, |struct_var_data| {
                struct_var_data.new_usage(line, UsageType::LValue);
            })
        }

        self.mut_var(assigned_to.to_string(), |l_value| {
            l_value.is_mut = true;
            l_value.new_usage(line, UsageType::LValue);
        });
    }

    pub fn ptr_assignment(&mut self, borrowed: &str, assigned_to: &str, line: LineNumber) {
        // WARNING If rvalue_ids.len() > 1 for ptr assignments then raw ptr
        self.assignment(assigned_to, vec![borrowed.to_string()], line);

        let new_reference = Rc::new(RefCell::new(Reference::new(borrowed, assigned_to, line)));

        let l_value = self.variables.get_mut(assigned_to).expect("Var not in ctx");
        l_value.points_to.push(new_reference.clone());
        l_value.is_mut = false;

        self.variables
            .entry(borrowed.to_string())
            .and_modify(|rvalue| rvalue.pointed_to.push(new_reference.clone()));
    }

    pub fn array_declaration(&mut self, id: &str, c_type: CType, _count: usize) {
        // TODO Figure out how to represent arrays as pointers to nothing
        // The current solution is to just represent them as variables with a special type that's
        // compatible with pointers
        let var_data = VarData::new(CType::Array(Box::new(c_type)), false, None, None);
        self.declaration(id, var_data);
    }

    // TODO Figure out how to recursively mark things as mutable
    /// `ptr_chain` arg in order [ptr2, ptr1, value]
    pub fn deref_assignment<T>(&mut self, ptr_chain: &mut T, line: LineNumber)
    where
        T: Iterator<Item = String>,
    {
        let top_ptr = ptr_chain.next().expect("No pointers in chain");
        let ptr_data = self.get_var(&top_ptr).clone(); // TODO :[
        assert!(ptr_data.is_ptr());
        println!("HERE\n{}", ptr_data.is_mut);

        if let Some(field_info) = &ptr_data.fieldof_struct {
            self.mut_struct(field_info.struct_id.clone(), |struct_data| {
                struct_data.mut_field(field_info.field_id.clone(), |field| {
                    field
                        .ptr_type
                        .iter_mut()
                        // NOTE Doesn't upgrade raw pointers
                        .filter(|ptr_type| **ptr_type == ReferenceType::ConstBorrowed)
                        .for_each(|ptr_type| *ptr_type = ReferenceType::MutBorrowed)
                });
            });
            let struct_var_id = top_ptr.split(".").nth(0).unwrap();
            self.mut_var(struct_var_id.to_string(), |struct_var_data| {
                struct_var_data.new_usage(line, UsageType::LValue);
            })
        }

        self.mut_var(top_ptr, |ptr_var| {
            assert!(ptr_var.is_ptr());

            ptr_var.new_usage(line, UsageType::LValue);
            ptr_var
                .current_reference_held()
                .unwrap()
                .borrow_mut()
                .set_mut();
        });

        ptr_chain.for_each(|var_id| {
            let var_data = self.get_var_mut(&var_id);
            var_data.set_mut();
            if let Some(reference) = var_data.current_reference_held() {
                reference.borrow_mut().set_mut();
            }
        });

        // NOTE We don't want to also assign to the sub_var here, because we're checking actual
        // literal usages, not cascading usages
        // (otherwise we'd always get a ValueMutSameLine error)
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

    pub fn get_struct_mut(&mut self, id: &str) -> &mut StructData {
        self.structs.get_mut(id).expect("Struct not in map")
    }

    pub fn get_var(&self, id: &str) -> &VarData {
        self.variables.get(id).expect("Var not in map")
    }

    pub fn get_var_mut(&mut self, id: &str) -> &mut VarData {
        self.variables
            .get_mut(id)
            .expect("Var (to mutate) not in map")
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

    /// Constructes a pointer chain upwards
    pub fn construct_ptr_chain_upwards(
        &self,
        root: String,
        total_depth: u8,
        max_depth: u8,
    ) -> Vec<String> {
        if total_depth == max_depth {
            return vec![];
        }
        let ptrs = &self
            .variables
            .get(&root)
            .as_ref()
            .expect("Root in construct ptr chain not found in map")
            .pointed_to;

        match ptrs.is_empty() {
            false => {
                let mut chain = self.construct_ptr_chain_upwards(
                    ptrs.last().unwrap().borrow().get_reference_to().to_string(),
                    total_depth + 1,
                    max_depth,
                );
                chain.push(root.to_string());
                chain
            }
            true => vec![root.to_string()],
        }
    }

    pub fn construct_ptr_chain_downwards(
        &self,
        root: String,
        total_depth: u8,
        max_depth: u8,
    ) -> Vec<String> {
        if total_depth == max_depth {
            return vec![];
        }

        println!();
        self.variables
            .iter()
            .for_each(|var_data| println!("{:?}\n", var_data));
        println!();
        println!("root: {root}");

        let ptrs = &self
            .variables
            .get(&root)
            .as_ref()
            .expect("Root in construct ptr chain not found in map")
            .points_to;

        match ptrs.is_empty() {
            false => {
                let mut chain = self.construct_ptr_chain_downwards(
                    ptrs.last().unwrap().borrow().get_reference_to().to_string(),
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
