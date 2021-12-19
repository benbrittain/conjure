use {
    super::error::Error,
    crate::shape::CsgFunc,
    std::{collections::HashMap, fmt, sync::Arc},
};

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum KeyTy {
    Str(String),
    Keyword(String),
}

#[derive(Clone)]
pub enum Ty {
    List(Vec<Ty>),
    Vector(Vec<Ty>),
    Number(f32),
    True,
    False,
    Symbol(String),
    Str(String),
    Keyword(String),
    HashMap(HashMap<KeyTy, Ty>),
    Function(fn(&[Ty]) -> Result<Ty, Error>),
    CsgFunc(Arc<CsgFunc>),
}

impl std::fmt::Debug for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::List(l) => f.debug_list().entries(l.iter()).finish(),
            Ty::Vector(v) => f.debug_struct("Vector").field("inside", v).finish(),
            Ty::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Ty::Symbol(s) => f.debug_tuple("Symbol").field(s).finish(),
            Ty::Keyword(s) => f.debug_tuple("Keyword").field(s).finish(),
            Ty::Str(s) => f.debug_tuple("String").field(s).finish(),
            Ty::False => f.debug_struct("False").finish(),
            Ty::True => f.debug_struct("True").finish(),
            Ty::Function(_) => f.debug_struct("<function>").finish(),
            Ty::CsgFunc(_) => f.debug_struct("<csg>").finish(),
            _ => panic!("DEBUG not implemented"),
        }
    }
}

impl TryInto<KeyTy> for Ty {
    type Error = Error;

    fn try_into(self) -> Result<KeyTy, Self::Error> {
        match self {
            Ty::Str(s) => Ok(KeyTy::Str(s)),
            Ty::Keyword(s) => Ok(KeyTy::Keyword(s)),
            t => Err(Error::InvalidKeyType(t)),
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", crate::lang::Reader::to_string(self, true))
    }
}
