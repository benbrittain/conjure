use {
    super::{error::Error, types::Ty},
    crate::shape::CsgFunc,
    std::collections::{hash_map::IntoIter, HashMap},
};

pub struct Namespace(HashMap<String, Ty>);

impl Namespace {
    pub fn new() -> Namespace {
        let mut ns = Namespace(HashMap::new());

        // addition
        ns.add_function("+", |list| {
            let mut accum = 0;
            for num in list {
                match num {
                    Ty::Number(n) => {
                        accum += n;
                    }
                    t => return Err(Error::InvalidType(t.clone())),
                }
            }
            Ok(Ty::Number(accum))
        });

        // subtraction
        ns.add_function("-", |list| match list {
            [Ty::Number(n), Ty::Number(m)] => Ok(Ty::Number(n - m)),
            _ => Err(Error::UnknownTypeCheck),
        });

        // CSG

        // csg union
        ns.add_function("union", |list| match list {
            [Ty::CsgFunc(fn1), Ty::CsgFunc(fn2)] => {
                let fn1 = fn1.clone();
                let fn2 = fn2.clone();
                let func = CsgFunc::new(Box::new(move |x, y, z| {
                    f32::min(fn1.call(x, y, z), fn2.call(x, y, z))
                }));
                Ok(Ty::CsgFunc(func))
            }
            _ => Err(Error::UnknownTypeCheck),
        });

        // csg intersection
        ns.add_function("intersect", |list| match list {
            [Ty::CsgFunc(fn1), Ty::CsgFunc(fn2)] => {
                let fn1 = fn1.clone();
                let fn2 = fn2.clone();
                let func = CsgFunc::new(Box::new(move |x, y, z| {
                    f32::max(fn1.call(x, y, z), fn2.call(x, y, z))
                }));
                Ok(Ty::CsgFunc(func))
            }
            _ => Err(Error::UnknownTypeCheck),
        });

        // csg sphere
        ns.add_function("sphere", |list| match list {
            [Ty::Number(n)] => {
                let radius = n.clone();
                let func = CsgFunc::new(Box::new(move |x, y, z| {
                    (((0.0 - z) * (0.0 - z)) + ((0.0 - x) * (0.0 - x)) + ((0.0 - y) * (0.0 - y)))
                        .sqrt()
                        - (radius as f32)
                }));
                Ok(Ty::CsgFunc(func))
            }
            _ => Err(Error::UnknownTypeCheck),
        });

        // csg cube
        ns.add_function("cube", |list| match list {
            [Ty::Vector(ll), Ty::Vector(ur)] => {
                let ll = ll.clone();
                let ur = ur.clone();
                let (ur_x, ur_y, ur_z) = match ur[..] {
                    [Ty::Number(x), Ty::Number(y), Ty::Number(z)] => (x as f32, y as f32, z as f32),
                    _ => return Err(Error::UnknownTypeCheck),
                };

                let (ll_x, ll_y, ll_z) = match ll[..] {
                    [Ty::Number(x), Ty::Number(y), Ty::Number(z)] => (x as f32, y as f32, z as f32),
                    _ => return Err(Error::UnknownTypeCheck),
                };

                let func = CsgFunc::new(Box::new(move |x, y, z| {
                    f32::max(
                        z - ur_z,
                        f32::max(
                            ll_z - z,
                            f32::max(ll_y - y, f32::max(y - ur_y, f32::max(ll_x - x, x - ur_x))),
                        ),
                    )
                }));
                Ok(Ty::CsgFunc(func))
            }
            _ => Err(Error::UnknownTypeCheck),
        });

        ns
    }

    pub fn add_function(&mut self, symbol: &str, func: for<'r> fn(&'r [Ty]) -> Result<Ty, Error>) {
        self.0.insert(symbol.to_string(), Ty::Function(func));
    }
}

impl IntoIterator for Namespace {
    type Item = (String, Ty);
    type IntoIter = IntoIter<String, Ty>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
