#[derive(PartialEq, Clone)]
pub enum TypeKind {
    TyNone,
    TyInt,
}

impl Default for TypeKind {
    fn default() -> Self {
        TypeKind::TyInt
    }
}

#[derive(Default)]
pub struct Type {
    pub kind: TypeKind,
}

impl Type {
    pub fn consume_type(s: &str) -> Self {
        let mut returntype = TypeKind::TyNone;
        if s == "int" {
            returntype = TypeKind::TyInt;
        }

        Self { kind: returntype }
    }
}
