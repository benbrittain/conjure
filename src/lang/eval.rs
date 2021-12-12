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
        Ty::List(ref _l) => {
            // TODO eval special forms here

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
