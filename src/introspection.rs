use std::collections::{HashMap, HashSet};

use crate::{
    parser::ast::{Expr, ExprVal},
    Node,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TeraIntrospection {
    pub extends: HashSet<String>,
    pub includes: HashSet<String>,
    pub macros: HashSet<String>,
    pub idents: HashSet<String>,
}

impl TeraIntrospection {
    pub fn new(nodes: &[Node], mapping: &mut HashMap<String, Vec<String>>) -> Self {
        let mut extends = HashSet::new();
        let mut includes = HashSet::new();
        let mut macros = HashSet::new();
        let mut idents = HashSet::new();
        let mut items = vec![];

        for node in nodes {
            match node {
                Node::Extends(_, name) => {
                    extends.insert(name.to_string());
                }
                Node::Include(_, template, _) => {
                    includes.insert(template.iter().join(""));
                }
                Node::ImportMacro(_, name, _) => {
                    macros.insert(name.to_string());
                }
                Node::Block(_, block, _) => {
                    items.push(TeraIntrospection::new(block.body.as_ref(), mapping))
                }
                Node::Forloop(_, for_loop, _) => {
                    let value = for_loop.value.clone();
                    add_mapping(mapping, &value, &for_loop.container, true);
                    add_ident(&mut idents, &for_loop.container, mapping);
                    items.push(TeraIntrospection::new(for_loop.body.as_ref(), mapping))
                }
                Node::If(if_cond, _) => {
                    for (_, expr, body) in &if_cond.conditions {
                        add_ident(&mut idents, expr, mapping);
                        items.push(TeraIntrospection::new(body.as_ref(), mapping))
                    }
                    if let Some(else_cond) = &if_cond.otherwise {
                        items.push(TeraIntrospection::new(else_cond.1.as_ref(), mapping))
                    }
                }
                Node::VariableBlock(_, expr) => {
                    add_ident(&mut idents, expr, mapping);
                }
                _ => (),
            }
        }

        let mut ins = Self {
            extends,
            includes,
            macros,
            idents,
        };

        for item in items {
            ins.merge(item);
        }

        ins
    }

    fn merge(&mut self, other: TeraIntrospection) {
        self.extends.extend(other.extends);
        self.includes.extend(other.includes);
        self.macros.extend(other.macros);
        self.idents.extend(other.idents);
    }
}

fn add_ident(idents: &mut HashSet<String>, expr: &Expr, mapping: &HashMap<String, Vec<String>>) {
    if let ExprVal::Ident(ref v) = expr.val {
        let names = split_ident(v);
        let name = expand_names(&names, mapping).into_iter().join(".");
        idents.insert(name);
    }
}

// use nom to parse a.b[c] or a[b][c] or a.b.c to ["a", "b", "c"]
fn split_ident(ident: &str) -> Vec<String> {
    ident
        .split(&['.', '['])
        .map(|v| v.trim_end_matches(']').to_string())
        .collect()
}

fn expand_names(names: &[String], mapping: &HashMap<String, Vec<String>>) -> Vec<String> {
    let first = names.first().expect("should exits");
    if let Some(v) = mapping.get(first) {
        let mut ret = v.clone();
        ret.extend_from_slice(&names[1..]);
        expand_names(&ret, mapping)
    } else {
        names.to_vec()
    }
}

fn add_mapping(
    mapping: &mut HashMap<String, Vec<String>>,
    value: &str,
    expr: &Expr,
    is_loop: bool,
) {
    if let ExprVal::Ident(ref v) = expr.val {
        let mut names = split_ident(v);
        if is_loop {
            if let Some(v) = names.last_mut() {
                v.push_str("()")
            }
        }
        mapping.insert(value.to_owned(), names);
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    use super::*;

    #[test]
    fn tera_introspection_should_work() {
        let extend = include_str!("../fixtures/table.j2");
        let nodes = parse(extend).unwrap();
        let mut mapping = HashMap::new();
        let ins = TeraIntrospection::new(nodes.as_ref(), &mut mapping);
        assert!(ins
            .extends
            .contains("crn:cws:cella:us-west2::view:todo/main"));
        assert!(ins
            .includes
            .contains("crn:cws:cella:us-west2::view:todo/table"));
        assert!(ins.macros.is_empty());
        assert_eq!(
            ins.idents,
            [
                "data.items().col",
                "data.items",
                "data.names",
                "config.edit_get",
                "loop.first",
                "data.items().values",
                "data.items().values().abc",
                "config.edit_title"
            ]
            .iter()
            .map(|v| v.to_string())
            .collect()
        );
    }
}
