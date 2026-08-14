

// src/workflows/enforcement.rs
//! Workflow enforcement layer - ensures agents follow mandatory workflow steps
//! 
//! This module provides enforcement for the required workflow:
//! 1. get_workflow - MUST be called first to retrieve workflow rules
//! 2. search_memory - MUST be called before any substantive action
//! 3. get_patterns - SHOULD be called for repetitive decisions
//! 4. Other tools - Only available after mandatory steps



mod enforcer;

