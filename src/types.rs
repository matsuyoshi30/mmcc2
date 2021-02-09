#[derive(PartialEq, Clone)]
pub enum TypeKind {
    TyNone,
    TyInt,
    TyPtr,
}

impl Default for TypeKind {
    fn default() -> Self {
        TypeKind::TyNone
    }
}

#[derive(Default, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
}
