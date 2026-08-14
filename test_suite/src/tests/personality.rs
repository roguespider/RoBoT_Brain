//! T1-10B-01 — migrated from `src/personality/mod.rs` `#[cfg(test)] mod tests`.
//!
//! The src/ unit tests (16 fns) exercised `Personality` methods directly.
//! test_suite cannot import robot_brain source, so each behavior is re-expressed
//! through the public MCP surface that invokes those methods:
//!   - `get_personality`            -> Personality::new defaults (preset, traits)
//!   - `apply_personality_preset`   -> Personality::apply_preset (valid + invalid)
//!   - `list_personality_presets`   -> Personality::list_presets
//!   - `set_personality_traits`     -> Personality::set_traits (trait change)
//!   - `get_personality_decision`   -> Personality::decide (approach/reason)
//!   - `format_response`            -> CommunicationStyle::format_response
//!   - `get_personality` comm_style -> Personality::get_communication_style
//!
//! Internal-only methods with NO MCP surface (adapt_from_experience,
//! should_explore, should_take_risk, should_use_creativity, get_timeout,
//! success_rate, and the clamping / preset->custom behavior of adjust_trait)
//! are NOT migrated — per the Group B decision they are removed from src/
//! (their methods remain in production; only the src/ unit tests are deleted).
//! decide() indirectly exercises should_explore/should_take_risk, so the
//! decision-making logic is still covered through get_personality_decision.
//!
//! Personality state is shared (server-wide App mutex). Each test resets to the
//! "balanced" preset first for deterministic state. Traits are f32, so asserts
//! use an approximate compare (abs diff < 0.01) rather than exact equality.

use crate::TestMcpClient;
use crate::TestStats;

/// Parse the JSON payload from a tool result's content[0].text.
fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let text = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no content text in tool result"))?;
    Ok(serde_json::from_str(text)?)
}

/// f32-stored traits compare with tolerance (avoid 0.7 vs 0.699999988079071).
fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

/// Reset personality to the "balanced" preset so tests start from known state.
async fn reset_to_balanced(client: &mut TestMcpClient) {
    let _ = client
        .call_tool("apply_personality_preset", serde_json::json!({ "preset": "balanced" }))
        .await;
}

pub async fn run_personality_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Personality defaults/preset/traits/decision (T1-10B-01) ---");

    // --- test_default_personality ---
    reset_to_balanced(client).await;
    let default_ok = match client.call_tool("get_personality", serde_json::json!({})).await {
        Ok(r) => payload_json(&r)
            .ok()
            .map(|v| {
                let preset = v.get("preset").and_then(|p| p.as_str()) == Some("balanced");
                let curiosity = v
                    .pointer("/traits/curiosity")
                    .and_then(|t| t.as_f64())
                    .map(|c| approx(c, 0.7))
                    .unwrap_or(false);
                preset && curiosity
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_personality(default) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if default_ok {
        crate::teeprintln!("  [OK] default personality: preset=balanced, curiosity=0.7 (Personality::new defaults)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] default personality: expected preset=balanced, curiosity=0.7");
        stats.failed += 1;
    }

    // --- test_apply_preset (valid) ---
    reset_to_balanced(client).await;
    let apply_ok = match client
        .call_tool("apply_personality_preset", serde_json::json!({ "preset": "analytical" }))
        .await
    {
        Ok(r) => {
            let applied = payload_json(&r)
                .ok()
                .and_then(|v| v.get("applied").and_then(|a| a.as_bool()))
                == Some(true);
            if !applied {
                false
            } else {
                match client.call_tool("get_personality", serde_json::json!({})).await {
                    Ok(g) => payload_json(&g)
                        .ok()
                        .map(|v| {
                            let preset = v.get("preset").and_then(|p| p.as_str()) == Some("analytical");
                            let caution = v
                                .pointer("/traits/caution")
                                .and_then(|t| t.as_f64())
                                .map(|c| approx(c, 0.8))
                                .unwrap_or(false);
                            let thorough = v
                                .pointer("/traits/thoroughness")
                                .and_then(|t| t.as_f64())
                                .map(|t| approx(t, 0.95))
                                .unwrap_or(false);
                            preset && caution && thorough
                        })
                        .unwrap_or(false),
                    Err(_) => false,
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] apply_personality_preset(analytical) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if apply_ok {
        crate::teeprintln!("  [OK] apply preset: analytical applied, caution=0.8, thoroughness=0.95 (Personality::apply_preset valid)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] apply preset: analytical not applied or traits wrong");
        stats.failed += 1;
    }

    // --- test_apply_invalid_preset ---
    reset_to_balanced(client).await;
    let invalid_ok = match client
        .call_tool("apply_personality_preset", serde_json::json!({ "preset": "nonexistent_preset_xyz" }))
        .await
    {
        Ok(r) => {
            let applied_false = payload_json(&r)
                .ok()
                .and_then(|v| v.get("applied").and_then(|a| a.as_bool()))
                == Some(false);
            if !applied_false {
                false
            } else {
                match client.call_tool("get_personality", serde_json::json!({})).await {
                    Ok(g) => payload_json(&g)
                        .ok()
                        .and_then(|v| {
                            v.get("preset").and_then(|p| p.as_str()).map(|s| s == "balanced")
                        })
                        .unwrap_or(false),
                    Err(_) => false,
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] apply_personality_preset(invalid) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if invalid_ok {
        crate::teeprintln!("  [OK] invalid preset: applied=false, preset stays balanced (Personality::apply_preset invalid)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] invalid preset: should report applied=false and leave preset unchanged");
        stats.failed += 1;
    }

    // --- test_list_presets ---
    reset_to_balanced(client).await;
    let list_ok = match client.call_tool("list_personality_presets", serde_json::json!({})).await {
        Ok(r) => payload_json(&r)
            .ok()
            .map(|v| {
                let presets = v.get("presets").and_then(|p| p.as_array());
                if let Some(arr) = presets {
                    let names: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
                    names.contains(&"balanced")
                        && names.contains(&"analytical")
                        && names.contains(&"creative")
                } else {
                    false
                }
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] list_personality_presets — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if list_ok {
        crate::teeprintln!("  [OK] list presets: balanced/analytical/creative present (Personality::list_presets)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] list presets: expected balanced/analytical/creative");
        stats.failed += 1;
    }

    // --- test_set_trait (trait change via set_personality_traits) ---
    reset_to_balanced(client).await;
    let set_ok = match client
        .call_tool(
            "set_personality_traits",
            serde_json::json!({ "curiosity": 0.9, "caution": 0.5, "creativity": 0.6, "patience": 0.7, "thoroughness": 0.8, "verbosity": 0.5, "risk_tolerance": 0.4, "humor_level": 0.3 }),
        )
        .await
    {
        Ok(r) => {
            let updated = payload_json(&r)
                .ok()
                .and_then(|v| v.get("updated").and_then(|u| u.as_bool()))
                == Some(true);
            if !updated {
                false
            } else {
                match client.call_tool("get_personality", serde_json::json!({})).await {
                    Ok(g) => payload_json(&g)
                        .ok()
                        .and_then(|v| {
                            v.pointer("/traits/curiosity").and_then(|t| t.as_f64()).map(|c| approx(c, 0.9))
                        })
                        .unwrap_or(false),
                    Err(_) => false,
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] set_personality_traits — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if set_ok {
        crate::teeprintln!("  [OK] set trait: curiosity set to 0.9 and reflected in get (Personality::set_traits)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] set trait: curiosity not updated to 0.9");
        stats.failed += 1;
    }

    // --- test_communication_style (verbosity -> communication_style) ---
    reset_to_balanced(client).await;
    let style_ok = {
        let mut all_match = true;
        for (verbosity, expect) in [(0.2_f64, "Concise"), (0.5, "Balanced"), (0.8, "Detailed")] {
            let _ = client
                .call_tool(
                    "set_personality_traits",
                    serde_json::json!({ "verbosity": verbosity, "caution": 0.5, "creativity": 0.6, "curiosity": 0.7, "patience": 0.7, "thoroughness": 0.8, "risk_tolerance": 0.4, "humor_level": 0.3 }),
                )
                .await;
            let got = client
                .call_tool("get_personality", serde_json::json!({}))
                .await
                .ok()
                .and_then(|r| payload_json(&r).ok())
                .and_then(|v| v.get("communication_style").and_then(|s| s.as_str()).map(String::from));
            if got.as_deref() != Some(expect) {
                all_match = false;
                crate::teeprintln!(
                    "  [FAIL] comm style: verbosity={} expected {} got {:?} (get_communication_style)",
                    verbosity, expect, got
                );
            }
        }
        all_match
    };
    if style_ok {
        crate::teeprintln!("  [OK] communication style: verbosity 0.2/0.5/0.8 -> Concise/Balanced/Detailed (get_communication_style)");
        stats.passed += 1;
    } else {
        stats.failed += 1;
    }

    // --- test_format_response (detailed vs concise length) ---
    reset_to_balanced(client).await;
    let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6";
    let fmt_ok = match (
        client.call_tool("format_response", serde_json::json!({ "content": content, "style": "detailed" })).await,
        client.call_tool("format_response", serde_json::json!({ "content": content, "style": "concise" })).await,
    ) {
        (Ok(d), Ok(co)) => {
            let detailed = payload_json(&d)
                .ok()
                .and_then(|v| v.get("formatted").and_then(|f| f.as_str()).map(String::from))
                .unwrap_or_default();
            let concise = payload_json(&co)
                .ok()
                .and_then(|v| v.get("formatted").and_then(|f| f.as_str()).map(String::from))
                .unwrap_or_default();
            // Detailed keeps all 6 lines; concise is shorter than detailed.
            detailed.lines().count() == 6 && concise.len() < detailed.len()
        }
        (Err(e), _) | (_, Err(e)) => {
            crate::teeprintln!("  [FAIL] format_response — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if fmt_ok {
        crate::teeprintln!("  [OK] format response: detailed keeps all lines, concise is shorter (CommunicationStyle::format_response)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] format response: detailed should keep all 6 lines and be longer than concise");
        stats.failed += 1;
    }

    // --- test_decide (get_personality_decision with cautious preset) ---
    reset_to_balanced(client).await;
    let _ = client
        .call_tool("apply_personality_preset", serde_json::json!({ "preset": "cautious" }))
        .await;
    let decide_ok = match client
        .call_tool(
            "get_personality_decision",
            serde_json::json!({
                "confidence": 0.3,
                "potential_gain": 0.8,
                "potential_loss": 0.2,
                "uncertainty": 0.6,
                "time_available": 60
            }),
        )
        .await
    {
        Ok(r) => payload_json(&r)
            .ok()
            .map(|v| {
                let reason = v.get("reason").and_then(|r| r.as_str()).unwrap_or("");
                let approach = v.get("approach").and_then(|a| a.as_str()).unwrap_or("");
                // cautious preset -> reason mentions "cautious"; a cautious preset
                // yields a Thorough approach.
                reason.contains("cautious") && approach == "Thorough"
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_personality_decision — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if decide_ok {
        crate::teeprintln!("  [OK] decide: cautious preset -> reason mentions cautious, approach Thorough (Personality::decide)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] decide: cautious preset should mention cautious and yield Thorough approach");
        stats.failed += 1;
    }

    // Reset state for any subsequent test modules sharing the server.
    reset_to_balanced(client).await;

    Ok(())
}
