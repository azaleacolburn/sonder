use crate::error::ErrType as ET;
use crate::parser::TokenNode as Node;

enum PtrType {
    ConstPtrConst,
    ConstPtrMut,
    MutPtrConst,
    MutPtrMut,

    ConstRef,
    MutRef,
}

struct Ptr {
    name: String,
    t: PtrType,
}

struct StackData {
    occurences: Vec<Node>,
    refs: Vec<Ptr>, // type_t: we don't care about how large data is
}

struct Function<'a> {
    ptr_params: Vec<&'a StackData>,
    owned_params: Vec<&'a StackData>,
}

struct Arena {
    data: Vec<StackData>,
}
