//! MIR functions and programs

use super::{NodeId, MirStmt};
use super::stmt::Param;
use super::expr::MirBlock;
use nevermind_type_checker::Type;

/// A MIR function
#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub body: MirBlock,
    pub return_type: Type,
    pub id: NodeId,
}

impl MirFunction {
    pub fn new(
        name: String,
        params: Vec<Param>,
        body: MirBlock,
        return_type: Type,
        id: NodeId,
    ) -> Self {
        Self {
            name,
            params,
            body,
            return_type,
            id,
        }
    }
}

/// A complete MIR program
#[derive(Debug, Default, Clone)]
pub struct MirProgram {
    pub statements: Vec<MirStmt>,
}

impl MirProgram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_statement(&mut self, stmt: MirStmt) {
        self.statements.push(stmt);
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = MirFunction> + '_ {
        self.statements.iter().filter_map(|stmt| {
            if let MirStmt::Function { id, name, params, body, return_type } = stmt {
                Some(MirFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    return_type: return_type.clone(),
                    id: *id,
                })
            } else {
                None
            }
        })
    }
}