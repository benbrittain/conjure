use {
    super::{error::Error, types::Ty},
    crate::shape::CsgFunc,
    std::collections::{hash_map::IntoIter, HashMap},
};

pub struct Namespace(HashMap<String, Ty>);

impl Namespace {
    pub fn new() -> Namespace {
        let mut hm = HashMap::new();

        hm.insert(
            String::from("+"),
            Ty::Function(|list| {
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
            }),
        );

        hm.insert(
            String::from("-"),
            Ty::Function(|list| match list {
                [Ty::Number(n), Ty::Number(m)] => Ok(Ty::Number(n - m)),
                _ => Err(Error::UnknownTypeCheck),
            }),
        );

        hm.insert(
            String::from("sphere"),
            Ty::Function(|list| match list {
                [Ty::Number(n)] => {
                    let radius = n.clone();
                    let func = CsgFunc::new(Box::new(move |x, y, z| {
                        (((0.0 - z) * (0.0 - z))
                            + ((0.0 - x) * (0.0 - x))
                            + ((0.0 - y) * (0.0 - y)))
                            .sqrt()
                            - (radius as f32)
                    }));
                    Ok(Ty::CsgFunc(func))
                }
                _ => Err(Error::UnknownTypeCheck),
            }),
        );

        Namespace(hm)
    }
}

impl IntoIterator for Namespace {
    type Item = (String, Ty);
    type IntoIter = IntoIter<String, Ty>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
