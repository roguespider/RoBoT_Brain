//! Design Principles — Centralized enforcement of architecture principles (Chapter 02).
//!
//! Chapter 02 supplies the twelve design principles validated by
//! [`enforcer::PrinciplesEnforcer`]. Chapter 24 (AI Contributor Operating
//! Agreement) supplies the architecture alignment rules
//! ([`alignment`]) and the contribution acceptance gate ([`acceptance`]) that
//! runs before a contribution is accepted.

pub mod acceptance;
pub mod alignment;
pub mod autonomous;
pub mod confidence_history;
pub mod diagnostics;
pub mod enforcer;
pub mod knowledge_graph;
pub mod trace;
