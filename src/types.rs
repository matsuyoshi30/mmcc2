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
    pub size: usize,
}

impl Type {
    fn new_type(kind: TypeKind, size: usize) -> Self {
        Self {
            kind: kind,
            size: size,
            ..Default::default()
        }
    }

    pub fn new_int() -> Self {
        Type::new_type(TypeKind::TyInt, 4)
    }

    pub fn pointer_to(self) -> Self {
        Type {
            kind: TypeKind::TyPtr,
            ptr_to: Some(Box::new(self)),
            size: 8,
        }
    }

    pub fn is_integer(&self) -> bool {
        self.kind == TypeKind::TyInt
    }
}
