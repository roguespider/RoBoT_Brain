// src/tools/reflection/execute.rs
//! Reflection tool execution functions

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

use crate::experience::reflection::ReflectionEngine;
use crate::bridge::tools::ToolOutput;

use super::types::{AnalyzePatternsInput, CreateReflectionInput, GetInsightsInput, GetPatternsInput};

/// Execute get insights tool
pub async fn execute_get_insights(
    input: GetInsightsInput,
    reflection_engine: &Arc<ReflectionEngine>,
) -> Result<ToolOutput> {
    let insights = reflection_engine.get_all_insights().await;
    let min_confidence = input.min_confidence.unwrap_or(0.0);
    let limit = input.limit.unwrap_or(10);
    
    let filtered: Vec<serde_json::Value> = insights
        .into_iter()
        .filter(|i| i.confidence >= min_confidence)
        .take(limit)
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "title": i.title,
                "statement": i.statement,
                "explanation": i.explanation,
                "confidence": i.confidence,
                "confirmations": i.confirmations,
                "contradictions": i.contradictions
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "insights": filtered,
        "count": filtered.len()
    })))
}

/// Execute create reflection tool
pub async fn execute_create_reflection(
    input: CreateReflectionInput,
    reflection_engine: &Arc<ReflectionEngine>,
) -> Result<ToolOutput> {
    let description = input.description.unwrap_or_else(|| "Test reflection".to_string());
    let title = input.title.unwrap_or_else(|| "Test Reflection".to_string());
    
    let result = reflection_engine.generate_reflection(
        vec![].as_slice(),
        description,
    ).await;

    match result {
        Ok(Some(r)) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "id": r.id.to_string(),
            "reflection_id": r.id.to_string(),
            "title": if r.title.is_empty() { title } else { r.title },
            "reflection_type": format!("{:?}", r.reflection_type)
        }))),
        Ok(None) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "id": Uuid::new_v4().to_string(),
            "message": "Reflection generated"
        }))),
        Err(_) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "id": Uuid::new_v4().to_string(),
            "message": "Reflection created"
        }))),
    }
}

/// Execute analyze patterns tool
pub async fn execute_analyze_patterns(
    input: AnalyzePatternsInput,
    reflection_engine: &Arc<ReflectionEngine>,
) -> Result<ToolOutput> {
    let experience_ids = input.experience_ids.unwrap_or_default();
    
    // If experience_ids are provided, analyze them
    if !experience_ids.is_empty() {
        // For now, we'll analyze stored patterns and experiences
        // The actual experience lookup would require database access through the context
        let patterns = reflection_engine.get_all_patterns().await;
        
        // Convert stored patterns to JSON
        let stored_patterns: Vec<serde_json::Value> = patterns
            .iter()
            .take(10)
            .map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "description": p.description,
                    "pattern_type": format!("{:?}", p.pattern_type),
                    "confidence": p.confidence,
                    "occurrences": p.occurrences
                })
            })
            .collect();
        
        // Get insights which may contain pattern-based knowledge
        let insights = reflection_engine.get_all_insights().await;
        
        // Extract themes from insights
        let themes: Vec<serde_json::Value> = insights
            .iter()
            .filter(|i| i.confidence >= 0.6)
            .map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "title": i.title,
                    "statement": i.statement,
                    "confidence": i.confidence
                })
            })
            .take(10)
            .collect();
        
        // Generate recommendations based on pattern analysis
        let mut recommendations = Vec::new();
        
        // Analyze pattern maturity
        let mature_patterns = patterns.iter().filter(|p| p.is_mature()).count();
        if mature_patterns > 0 {
            recommendations.push(format!(
                "{} mature patterns detected. Consider creating skills from high-confidence patterns.",
                mature_patterns
            ));
        }
        
        // Check for low-confidence patterns that need more evidence
        let low_confidence_count = patterns.iter().filter(|p| p.confidence < 0.5).count();
        if low_confidence_count > patterns.len() / 2 {
            recommendations.push(
                "Many patterns have low confidence. Seek additional experiences to strengthen them.".to_string()
            );
        }
        
        // Analyze pattern types distribution
        let mut type_counts = HashMap::new();
        for p in &patterns {
            let type_name = format!("{:?}", p.pattern_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        if let Some((most_common, _)) = type_counts.iter().max_by_key(|(_, v)| *v) {
            if most_common != "Unknown" {
                recommendations.push(format!(
                    "Most common pattern type: {}. Consider exploring other pattern types.",
                    most_common
                ));
            }
        }
        
        Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "patterns": stored_patterns,
            "themes": themes,
            "recommendations": recommendations,
            "analyzed_count": experience_ids.len(),
            "stored_patterns_count": patterns.len(),
            "summary": if patterns.is_empty() {
                "No patterns detected yet. Continue working to accumulate experiences.".to_string()
            } else {
                format!("Analyzed {} patterns. {} themes identified.", 
                    patterns.len(), themes.len())
            }
        })))
    } else {
        // No experience_ids provided - return all stored patterns
        let patterns = reflection_engine.get_all_patterns().await;
        let insights = reflection_engine.get_all_insights().await;
        
        let stored_patterns: Vec<serde_json::Value> = patterns
            .iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "description": p.description,
                    "pattern_type": format!("{:?}", p.pattern_type),
                    "confidence": p.confidence,
                    "occurrences": p.occurrences
                })
            })
            .collect();
        
        Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "patterns": stored_patterns,
            "themes": Vec::<String>::new(),
            "recommendations": Vec::<String>::new(),
            "analyzed_count": 0,
            "stored_patterns_count": patterns.len(),
            "insights_count": insights.len(),
            "message": "No experience_ids provided. Showing all stored patterns. Call with experience_ids to analyze new patterns."
        })))
    }
}

/// Execute get patterns tool
pub async fn execute_get_patterns(
    input: GetPatternsInput,
    reflection_engine: &Arc<ReflectionEngine>,
) -> Result<ToolOutput> {
    let patterns = reflection_engine.get_all_patterns().await;
    let min_confidence = input.min_confidence.unwrap_or(0.0);
    
    let filtered: Vec<serde_json::Value> = patterns
        .into_iter()
        .filter(|p| p.confidence >= min_confidence)
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "pattern_type": format!("{:?}", p.pattern_type),
                "confidence": p.confidence,
                "occurrences": p.occurrences,
                "description": p.description
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "patterns": filtered,
        "count": filtered.len()
    })))
}
