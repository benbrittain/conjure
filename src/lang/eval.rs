use super::{environment::Env, error::Error, types::Ty};

fn eval_ast(ast: Ty, env: &Env) -> Result<Ty, Error> {
    match ast {
        Ty::Symbol(s) => env.lookup_sym(s),
        Ty::List(l) => Ok(Ty::List(l.into_iter().map(|elem| eval(elem, env).unwrap()).collect())),
        _ => Ok(ast),
    }
}

pub fn eval(ast: Ty, env: &Env) -> Result<Ty, Error> {
    match ast {
        Ty::List(ref list) => {
            // Evaluate special forms
            if let Ty::Symbol(s) = &list[0] {
                match s.as_str() {
                    "let" => {
                        return match &list[1] {
                            Ty::Vector(_) => {
                                let new_env = env.new_env(list[1].clone())?;
                                eval(list[2].clone(), &new_env)
                            }
                            _ => Err(Error::InvalidLetBinding),
                        }
                    }
                    _ => {}
                };
            }

            let evaled_ast = eval_ast(ast, env)?;
            match evaled_ast {
                Ty::List(ref l) => {
                    // apply the function `f` on the arguments in the list
                    if let Ty::Function(f) = &l[0] {
                        return f(&l[1..]);
                    }
                    Ok(evaled_ast)
                }
                _ => Err(Error::UnknownEval(evaled_ast)),
            }
        }
        e => eval_ast(e, env),
    }
}
