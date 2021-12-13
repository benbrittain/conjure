use {
    super::{error::Error, types::Ty},
    std::{cell::RefCell, collections::HashMap, rc::Rc},
};

#[derive(Clone)]
pub struct Env {
    // TODO Rc/Refcell is a bit lazy, consider doing something
    // more performant
    lookup: RefCell<HashMap<String, Ty>>,
    parent: Option<Rc<Env>>,
}

impl Env {
    /// Checks if a symbol is present in the current Environment
    pub fn lookup_sym(&self, key: String) -> Result<Ty, Error> {
        match self.lookup.borrow().get(&key) {
            Some(e) => Ok(e.clone()),
            None => match &self.parent {
                Some(p) => p.lookup_sym(key),
                None => Err(Error::UnknownSymbol(key)),
            },
        }
    }

    /// Registers a symbol in the current Environment
    pub fn register_sym(&self, key: String, ty: Ty) {
        self.lookup.borrow_mut().insert(key, ty);
    }

    /// Creates a child environment with it's own scope
    pub fn new_env(&self, ty: Ty) -> Result<Rc<Env>, Error> {
        let lookup = HashMap::new();
        let env =
            Rc::new(Env { lookup: RefCell::new(lookup), parent: Some(Rc::new(self.clone())) });
        match ty {
            Ty::Vector(l) => {
                for pairs in l.chunks(2) {
                    if let [Ty::Symbol(sym), val] = pairs {
                        env.register_sym(sym.clone(), super::eval::eval(val.clone(), &env)?);
                    } else {
                        return Err(Error::InvalidLetBinding);
                    }
                }
                Ok(env)
            }
            _ => Err(Error::InvalidLetBinding),
        }
    }

    /// New environment with nothing in scope
    pub fn new() -> Env {
        Env { lookup: RefCell::new(HashMap::new()), parent: None }
    }
}
