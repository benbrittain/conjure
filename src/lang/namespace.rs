use {
    super::{error::Error, types::Ty},
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
