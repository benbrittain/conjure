use {
    super::{error::Error, types::Ty},
    std::{cell::RefCell, collections::HashMap, rc::Rc},
};

pub struct Env {
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

    //pub fn new_env(self: &Rc<Self>, ty: Ty) -> Result<Rc<Env>, Error> {
    //    let lookup = HashMap::new();
    //    let env = Rc::new(Env {
    //            lookup: RefCell::new(lookup),
    //            parent: Some(self.clone()),
    //    });
    //    match ty {
    //        Ty::List(l) => {
    //            for pairs in l.chunks(2) {
    //                if let [Ty::Symbol(sym), val] = pairs {
    //                    env.register_sym(sym.clone(), eval::eval(val.clone(), &env)?);
    //                } else {
    //                    return Err(anyhow!("invalid env: {:?}", pairs));
    //                }
    //            }
    //            Ok(env)
    //        },
    //        Ty::Vector(l) => {
    //            for pairs in l.chunks(2) {
    //                if let [Ty::Symbol(sym), val] = pairs {
    //                    env.register_sym(sym.clone(), eval::eval(val.clone(), &env)?);
    //                } else {
    //                    return Err(anyhow!("invalid env: {:?}", pairs));
    //                }
    //            }
    //            Ok(env)
    //        },
    //        t => Err(anyhow!("Not a valid environment: {:?}", t))
    //    }
    //}

    pub fn new() -> Env {
        Env { lookup: RefCell::new(HashMap::new()), parent: None }
    }
}
