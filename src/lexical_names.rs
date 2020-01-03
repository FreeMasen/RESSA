use super::{Error, Position, Res};
use hash_chain::ChainMap;
use resast::prelude::*;
use std::borrow::Cow;
type LexMap<'a> = ChainMap<Cow<'a, str>, ()>;
type LexMap2<'a> = ChainMap<Cow<'a, str>, Position>;
pub enum DeclKind {
    Lex,
    Var,
    Func,
}

pub struct DuplicateNameDetector<'a> {
    lex: LexMap2<'a>,
    var: LexMap2<'a>,
    func: LexMap2<'a>,
}

impl<'a> DuplicateNameDetector<'a> {
    pub fn declare(&mut self, i: &Cow<'a, str>, kind: DeclKind, pos: Position) -> Res<()> {
        match kind {
            DeclKind::Lex => {
                self.check_var(i, pos)?;
                self.check_func(i, pos)?;
                self.add_lex(i, pos)
            }
            DeclKind::Var => unimplemented!(),
            DeclKind::Func => unimplemented!(),
        }
    }
    fn check_var(&self, i: &Cow<'a, str>, pos: Position) -> Res<()> {
        check(&self.var, i, pos)
    }
    fn add_var(&mut self, i: &Cow<'a, str>, pos: Position) -> Res<()> {
        add(&mut self.var, i, pos)
    }

    fn check_func(&self, i: &Cow<'a, str>, pos: Position) -> Res<()> {
        check(&self.func, i, pos)
    }
    fn add_func(&mut self, i: &Cow<'a, str>, pos: Position) -> Res<()> {
        add(&mut self.func, i, pos)
    }
    fn check_lex(&self, i: &Cow<'a, str>, pos: Position) -> Res<()> {
        check(&self.lex, i, pos)
    }
    fn add_lex(&mut self, i: &Cow<'a, str>, pos: Position) -> Res<()> {
        add(&mut self.lex, i, pos)
    }
}

pub fn check_for_ident<'a>(map: &LexMap<'a>, i: &Ident<'a>, start: Position) -> Res<()> {
    if map.get(&i.name).is_some() {
        Err(Error::LexicalRedecl(
            start,
            format!("{} was previously declared", i.name),
        ))
    } else {
        Ok(())
    }
}
fn check<'a>(map: &LexMap2<'a>, i: &Cow<'a, str>, pos: Position) -> Res<()> {
    if let Some(old_pos) = map.get(i) {
        Err(Error::LexicalRedecl(
            pos,
            format!("{} was previously declared ({})", i, old_pos),
        ))
    } else {
        Ok(())
    }
}

pub fn add_ident<'a>(map: &mut LexMap<'a>, i: &Ident<'a>, start: Position) -> Res<()> {
    if map.insert(i.name.clone(), ()).is_some() {
        Err(Error::LexicalRedecl(
            start,
            format!("{} was previously declared", i.name),
        ))
    } else {
        Ok(())
    }
}
pub fn add<'a>(map: &mut LexMap2<'a>, i: &Cow<'a, str>, start: Position) -> Res<()> {
    if map.insert(i.clone(), start).is_some() {
        Err(Error::LexicalRedecl(
            start,
            format!("{} was previously declared", i),
        ))
    } else {
        Ok(())
    }
}
pub fn add_pat<'a>(map: &mut LexMap<'a>, pat: &Pat<'a>, start: Position) -> Res<()> {
    match pat {
        Pat::Ident(ref i) => {
            log::trace!("add_pat ident {:?}", i.name);
            add_ident(map, i, start)
        }
        Pat::Array(ref a) => {
            for part in a {
                if let Some(ref i) = part {
                    match i {
                        ArrayPatPart::Expr(ex) => add_expr(map, ex, start)?,
                        ArrayPatPart::Pat(pat) => add_pat(map, pat, start)?,
                    }
                }
            }
            Ok(())
        }
        Pat::Assign(ref a) => add_pat(map, &*a.left, start),
        Pat::Obj(ref o) => {
            for part in o {
                match part {
                    ObjPatPart::Assign(ref prop) => match prop.key {
                        PropKey::Expr(ref ex) => add_expr(map, ex, start)?,
                        PropKey::Pat(ref pat) => add_pat(map, pat, start)?,
                        PropKey::Lit(ref _lit) => unreachable!("literal as identifier"),
                    },
                    ObjPatPart::Rest(ref pat) => add_pat(map, pat, start)?,
                }
            }
            Ok(())
        }
        Pat::RestElement(ref r) => add_pat(map, &*r, start),
    }
}

pub fn add_expr<'a>(map: &mut LexMap<'a>, expr: &Expr<'a>, start: Position) -> Res<()> {
    if let Expr::Ident(ref i) = expr {
        log::trace!("add_expr ident {:?}", i.name);
        add_ident(map, i, start)
    } else {
        Ok(())
    }
}
