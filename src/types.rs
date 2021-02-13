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

#[derive(Default, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
}

impl Type {
    pub fn pointer_to(self) -> Self {
        Type {
            kind: TypeKind::TyPtr,
            ptr_to: Some(Box::new(self)),
        }
    }

    pub fn is_integer(&self) -> bool {
        self.kind == TypeKind::TyInt
    }
}
