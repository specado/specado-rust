//! Capability comparison and lossiness detection

use serde::{Deserialize, Serialize};
use crate::capabilities::Capability;

/// Result of comparing two capability sets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityComparison {
    /// Capabilities present in source but not in target
    pub missing_capabilities: Vec<String>,
    
    /// Capabilities present in both but with different constraints
    pub constrained_capabilities: Vec<ConstraintDifference>,
    
    /// Capabilities present in target but not in source
    pub additional_capabilities: Vec<String>,
    
    /// Overall lossiness report
    pub lossiness_report: LossinessReport,
}

/// Difference in constraints between capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintDifference {
    pub capability: String,
    pub source_value: serde_json::Value,
    pub target_value: serde_json::Value,
    pub impact: LossinessImpact,
}

/// Report on potential data loss during transformation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossinessReport {
    /// Whether transformation would be lossy
    pub is_lossy: bool,
    
    /// Types of lossiness detected
    pub lossiness_types: Vec<LossinessType>,
    
    /// Severity of potential data loss
    pub severity: LossinessSeverity,
    
    /// Detailed description of potential losses
    pub details: Vec<String>,
    
    /// Recommendations for handling lossiness
    pub recommendations: Vec<String>,
}

/// Types of lossiness that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LossinessType {
    /// Feature not supported by target
    MissingFeature(String),
    
    /// Modality not supported by target
    MissingModality(String),
    
    /// Parameter range more restricted in target
    ConstrainedParameter(String),
    
    /// Role not supported by target
    MissingRole(String),
    
    /// Token limit exceeded
    TokenLimitExceeded,
    
    /// Rate limit more restrictive
    RateLimitReduced,
    
    /// Format not supported
    UnsupportedFormat(String),
    
    /// Custom/extension feature missing
    MissingExtension(String),
}

/// Severity levels for lossiness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LossinessSeverity {
    /// No data loss
    None,
    
    /// Minor loss that likely won't affect functionality
    Low,
    
    /// Moderate loss that may affect some functionality
    Medium,
    
    /// Significant loss that will affect core functionality
    High,
    
    /// Critical loss that makes transformation impossible
    Critical,
}

/// Impact level of constraint differences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LossinessImpact {
    None,
    Minor,
    Major,
}

impl CapabilityComparison {
    /// Compare source capabilities against target capabilities
    pub fn compare(source: &Capability, target: &Capability) -> Self {
        let mut missing_capabilities = Vec::new();
        let mut constrained_capabilities = Vec::new();
        let additional_capabilities = Vec::new();
        let mut lossiness_types = Vec::new();
        let mut details = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check model features
        Self::compare_features(source, target, &mut missing_capabilities, &mut lossiness_types, &mut details);
        
        // Check modalities
        Self::compare_modalities(source, target, &mut missing_capabilities, &mut lossiness_types, &mut details);
        
        // Check parameters
        Self::compare_parameters(source, target, &mut constrained_capabilities, &mut lossiness_types, &mut details);
        
        // Check roles
        Self::compare_roles(source, target, &mut missing_capabilities, &mut lossiness_types, &mut details);
        
        // Check constraints
        Self::compare_constraints(source, target, &mut constrained_capabilities, &mut lossiness_types, &mut details);
        
        // Generate recommendations
        Self::generate_recommendations(&lossiness_types, &mut recommendations);
        
        // Determine severity
        let severity = Self::determine_severity(&lossiness_types);
        let is_lossy = !lossiness_types.is_empty();
        
        Self {
            missing_capabilities,
            constrained_capabilities,
            additional_capabilities,
            lossiness_report: LossinessReport {
                is_lossy,
                lossiness_types,
                severity,
                details,
                recommendations,
            },
        }
    }
    
    fn compare_features(
        source: &Capability,
        target: &Capability,
        missing: &mut Vec<String>,
        lossiness: &mut Vec<LossinessType>,
        details: &mut Vec<String>,
    ) {
        if source.features.function_calling && !target.features.function_calling {
            missing.push("function_calling".to_string());
            lossiness.push(LossinessType::MissingFeature("function_calling".to_string()));
            details.push("Target provider does not support function calling".to_string());
        }
        
        if source.features.json_mode && !target.features.json_mode {
            missing.push("json_mode".to_string());
            lossiness.push(LossinessType::MissingFeature("json_mode".to_string()));
            details.push("Target provider does not support JSON mode for structured output".to_string());
        }
        
        if source.features.streaming && !target.features.streaming {
            missing.push("streaming".to_string());
            lossiness.push(LossinessType::MissingFeature("streaming".to_string()));
            details.push("Target provider does not support streaming responses".to_string());
        }
        
        if source.features.vision && !target.features.vision {
            missing.push("vision".to_string());
            lossiness.push(LossinessType::MissingFeature("vision".to_string()));
            details.push("Target provider does not support vision/image analysis".to_string());
        }
        
        if source.features.tool_use && !target.features.tool_use {
            if !target.features.function_calling {
                missing.push("tool_use".to_string());
                lossiness.push(LossinessType::MissingFeature("tool_use".to_string()));
                details.push("Target provider does not support tool use or function calling".to_string());
            }
        }
    }
    
    fn compare_modalities(
        source: &Capability,
        target: &Capability,
        missing: &mut Vec<String>,
        lossiness: &mut Vec<LossinessType>,
        details: &mut Vec<String>,
    ) {
        for modality in &source.modalities.input {
            if !target.modalities.input.contains(modality) {
                let modality_str = format!("{:?}", modality);
                missing.push(format!("input_{}", modality_str));
                lossiness.push(LossinessType::MissingModality(modality_str.clone()));
                details.push(format!("Target does not support {} input", modality_str));
            }
        }
        
        for modality in &source.modalities.output {
            if !target.modalities.output.contains(modality) {
                let modality_str = format!("{:?}", modality);
                missing.push(format!("output_{}", modality_str));
                lossiness.push(LossinessType::MissingModality(modality_str.clone()));
                details.push(format!("Target does not support {} output", modality_str));
            }
        }
    }
    
    fn compare_parameters(
        source: &Capability,
        target: &Capability,
        constrained: &mut Vec<ConstraintDifference>,
        lossiness: &mut Vec<LossinessType>,
        details: &mut Vec<String>,
    ) {
        // Check temperature support
        if source.parameters.temperature.supported && !target.parameters.temperature.supported {
            lossiness.push(LossinessType::ConstrainedParameter("temperature".to_string()));
            details.push("Target does not support temperature parameter".to_string());
        }
        
        // Check max_tokens
        if let (Some(source_max), Some(target_max)) = (
            source.parameters.max_tokens.max,
            target.parameters.max_tokens.max,
        ) {
            if target_max < source_max {
                constrained.push(ConstraintDifference {
                    capability: "max_tokens".to_string(),
                    source_value: serde_json::json!(source_max),
                    target_value: serde_json::json!(target_max),
                    impact: LossinessImpact::Major,
                });
                lossiness.push(LossinessType::ConstrainedParameter("max_tokens".to_string()));
                details.push(format!(
                    "Target max_tokens ({}) is less than source ({})",
                    target_max, source_max
                ));
            }
        }
    }
    
    fn compare_roles(
        source: &Capability,
        target: &Capability,
        missing: &mut Vec<String>,
        lossiness: &mut Vec<LossinessType>,
        details: &mut Vec<String>,
    ) {
        if source.roles.system && !target.roles.system {
            missing.push("system_role".to_string());
            lossiness.push(LossinessType::MissingRole("system".to_string()));
            details.push("Target does not support system role for instructions".to_string());
        }
        
        if source.roles.function && !target.roles.function {
            missing.push("function_role".to_string());
            lossiness.push(LossinessType::MissingRole("function".to_string()));
            details.push("Target does not support function role for function results".to_string());
        }
    }
    
    fn compare_constraints(
        source: &Capability,
        target: &Capability,
        constrained: &mut Vec<ConstraintDifference>,
        lossiness: &mut Vec<LossinessType>,
        details: &mut Vec<String>,
    ) {
        // Check context window
        if let (Some(source_ctx), Some(target_ctx)) = (
            source.constraints.tokens.max_context_window,
            target.constraints.tokens.max_context_window,
        ) {
            if target_ctx < source_ctx {
                constrained.push(ConstraintDifference {
                    capability: "context_window".to_string(),
                    source_value: serde_json::json!(source_ctx),
                    target_value: serde_json::json!(target_ctx),
                    impact: LossinessImpact::Major,
                });
                lossiness.push(LossinessType::TokenLimitExceeded);
                details.push(format!(
                    "Target context window ({}) is smaller than source ({})",
                    target_ctx, source_ctx
                ));
            }
        }
        
        // Check rate limits
        if let (Some(source_rpm), Some(target_rpm)) = (
            source.constraints.rate_limits.requests_per_minute,
            target.constraints.rate_limits.requests_per_minute,
        ) {
            if target_rpm < source_rpm {
                lossiness.push(LossinessType::RateLimitReduced);
                details.push(format!(
                    "Target rate limit ({} req/min) is lower than source ({} req/min)",
                    target_rpm, source_rpm
                ));
            }
        }
    }
    
    fn generate_recommendations(
        lossiness_types: &[LossinessType],
        recommendations: &mut Vec<String>,
    ) {
        let has_function_calling = lossiness_types.iter().any(|t| {
            matches!(t, LossinessType::MissingFeature(f) if f == "function_calling")
        });
        
        if has_function_calling {
            recommendations.push(
                "Consider using tool_use format if target supports it, or restructure request without functions".to_string()
            );
        }
        
        let has_token_limit = lossiness_types.iter().any(|t| {
            matches!(t, LossinessType::TokenLimitExceeded)
        });
        
        if has_token_limit {
            recommendations.push(
                "Consider chunking the input or using a model with larger context window".to_string()
            );
        }
        
        let has_modality_loss = lossiness_types.iter().any(|t| {
            matches!(t, LossinessType::MissingModality(_))
        });
        
        if has_modality_loss {
            recommendations.push(
                "Consider preprocessing multimodal inputs or using a multimodal-capable provider".to_string()
            );
        }
    }
    
    fn determine_severity(lossiness_types: &[LossinessType]) -> LossinessSeverity {
        if lossiness_types.is_empty() {
            return LossinessSeverity::None;
        }
        
        let mut max_severity = LossinessSeverity::Low;
        
        for loss_type in lossiness_types {
            let severity = match loss_type {
                LossinessType::MissingFeature(f) => {
                    match f.as_str() {
                        "function_calling" | "tool_use" => LossinessSeverity::High,
                        "json_mode" => LossinessSeverity::Medium,
                        "streaming" => LossinessSeverity::Low,
                        _ => LossinessSeverity::Medium,
                    }
                }
                LossinessType::MissingModality(_) => LossinessSeverity::Critical,
                LossinessType::TokenLimitExceeded => LossinessSeverity::High,
                LossinessType::ConstrainedParameter(_) => LossinessSeverity::Medium,
                LossinessType::MissingRole(r) => {
                    if r == "system" {
                        LossinessSeverity::Medium
                    } else {
                        LossinessSeverity::Low
                    }
                }
                LossinessType::RateLimitReduced => LossinessSeverity::Low,
                LossinessType::UnsupportedFormat(_) => LossinessSeverity::High,
                LossinessType::MissingExtension(_) => LossinessSeverity::Low,
            };
            
            if severity > max_severity {
                max_severity = severity;
            }
        }
        
        max_severity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_lossiness() {
        let cap1 = Capability::default();
        let cap2 = Capability::default();
        
        let comparison = CapabilityComparison::compare(&cap1, &cap2);
        assert!(!comparison.lossiness_report.is_lossy);
        assert_eq!(comparison.lossiness_report.severity, LossinessSeverity::None);
    }
    
    #[test]
    fn test_missing_feature_lossiness() {
        let mut source = Capability::default();
        source.features.function_calling = true;
        source.features.json_mode = true;
        
        let target = Capability::default();
        
        let comparison = CapabilityComparison::compare(&source, &target);
        assert!(comparison.lossiness_report.is_lossy);
        assert_eq!(comparison.missing_capabilities.len(), 2);
        assert!(comparison.lossiness_report.severity >= LossinessSeverity::Medium);
    }
    
    #[test]
    fn test_constraint_lossiness() {
        let mut source = Capability::default();
        source.constraints.tokens.max_context_window = Some(10000);
        
        let mut target = Capability::default();
        target.constraints.tokens.max_context_window = Some(5000);
        
        let comparison = CapabilityComparison::compare(&source, &target);
        assert!(comparison.lossiness_report.is_lossy);
        assert!(comparison.lossiness_report.lossiness_types.contains(&LossinessType::TokenLimitExceeded));
    }
}