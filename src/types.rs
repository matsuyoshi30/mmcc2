#[derive(PartialEq, Clone)]
pub enum TypeKind {
    TyNone,
    TyInt,
    TyPtr,
    TyArr,
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
    pub size_array: usize,
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
            ..Default::default()
        }
    }

    pub fn array_of(self, n: usize) -> Self {
        Self {
            kind: TypeKind::TyArr,
            size: self.size * n,
            ptr_to: Some(Box::new(self)),
            size_array: n,
        }
    }

    pub fn is_integer(&self) -> bool {
        self.kind == TypeKind::TyInt
    }
}
