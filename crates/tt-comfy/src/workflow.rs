//! Tlatoāni Tales — ComfyUI workflow JSON generator.
//!
//! This module is the *upstream* side of the seam described on
//! [`crate::Workflow`]. The pass-through wrapper ferries JSON across the
//! trust boundary; this module produces the JSON in the first place from
//! spec-level inputs (the panel's positive / negative prompts, its LoRAs,
//! its seed, its output prefix). One panel-spec → one workflow JSON →
//! one `/prompt` submit → one PNG.
//!
//! The shape we emit is the canonical ComfyUI node-graph envelope:
//!
//! ```jsonc
//! {
//!   "1": { "class_type": "CheckpointLoaderSimple", "inputs": { ... } },
//!   "2": { "class_type": "LoraLoader",             "inputs": { ... } },
//!   ...
//! }
//! ```
//!
//! Node IDs are numeric strings in ascending order. The order of keys in
//! the serialized JSON is stable (`serde_json::Map` is backed by
//! `BTreeMap`, so identical inputs → byte-identical output — see
//! `build_workflow_is_byte_deterministic` below). Byte-identical serialized
//! JSON is what lets this artefact act as the cache-key-bearing witness
//! promised by `@Lesson S1-500` — two edits that happen to normalize to
//! the same spec-level inputs reconcile to the same workflow on disk, and
//! thus to the same `panel_hash`.
//!
//! Governing specs: `openspec/specs/orchestrator/spec.md` §tt-comfy,
//! `openspec/specs/character-loras/spec.md` §Trigger tokens,
//! `openspec/specs/visual-qa-loop/spec.md` §Reroll addendum.
//!
// @trace spec:orchestrator, spec:character-loras, spec:visual-qa-loop
// @Lesson S1-500

use serde_json::{json, Map, Value};
use unicode_normalization::UnicodeNormalization;

use crate::Workflow;

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// Per-panel prompt inputs that feed a single workflow.
///
/// Borrowed strings, because callers already have the text in hand
/// (panel-<N>.prompt.md parsed upstream) and we only need to weave it
/// into the workflow JSON. `addendum` is set by `tt-qa::derive_addendum`
/// on reroll — empty on the first render.
// @trace spec:visual-qa-loop
#[derive(Debug, Clone)]
pub struct PanelPrompt<'a> {
    /// The strip's per-panel positive prompt copy, verbatim.
    pub positive: &'a str,
    /// The strip's per-panel negative prompt copy, verbatim.
    pub negative: &'a str,
    /// Reroll addendum appended to the positive prompt with a leading
    /// `, ` when present. Produced by `tt-qa` from the failed visual-QA
    /// checks; `None` on the first render.
    pub addendum: Option<&'a str>,
    /// Target image width in pixels.
    pub width: u32,
    /// Target image height in pixels.
    pub height: u32,
    /// Deterministic RNG seed for this panel — lives in
    /// `panel-<N>.prompt.md` frontmatter and is already part of the
    /// `panel_hash` via `tt-hashing::panel_hash`.
    pub seed: u64,
}

/// One character LoRA chained into the workflow.
///
/// `filename` is the on-disk name inside the untrusted container's
/// `tools/loras/` bind mount (e.g. `"tlatoani-v1.safetensors"`).
/// `trigger` is the PascalCase trigger token from the character's
/// `lora-manifest.json` (e.g. `"TlhAxolotlSage"` or `"CoviFigure"`).
/// Strengths default to `1.0`; overrides are routed through the usual
/// spec-mutation primitive rather than freeform knobs.
// @trace spec:character-loras
#[derive(Debug, Clone)]
pub struct CharacterLora<'a> {
    /// LoRA filename inside `tools/loras/` (ASCII-only per TB03).
    pub filename: &'a str,
    /// PascalCase trigger token — prepended to the positive prompt.
    pub trigger: &'a str,
    /// LoRA model-side strength. Default `1.0`.
    pub model_strength: f32,
    /// LoRA CLIP-side strength. Default `1.0`.
    pub clip_strength: f32,
}

/// Full spec-level description of a workflow.
///
/// Defaults tuned for FLUX.1-schnell are exposed via
/// [`WorkflowSpec::flux_schnell_default`]; caller overrides are
/// explicit. The defaults matter: schnell is a distilled model, so
/// `cfg=1.0` and `steps=4` are *correctness* knobs, not performance
/// knobs — a `cfg` above ~1.5 or a step count above ~8 actively harms
/// output quality on schnell. See `orchestrator/spec.md` for the
/// reproducibility promise those defaults are part of.
// @trace spec:orchestrator, spec:character-loras, spec:visual-qa-loop
#[derive(Debug, Clone)]
pub struct WorkflowSpec<'a> {
    /// Base checkpoint filename (e.g. `"flux1-schnell-fp8.safetensors"`).
    pub base_checkpoint: &'a str,
    /// Per-character LoRAs, rendered as a chain of `LoraLoader` nodes.
    /// Empty for panels with no character LoRAs (e.g. a silent
    /// establishing shot).
    pub loras: Vec<CharacterLora<'a>>,
    /// Per-panel prompt, size, seed.
    pub panel: PanelPrompt<'a>,
    /// `filename_prefix` passed through to `SaveImage` (e.g. `"tt-01-p1"`).
    pub output_prefix: &'a str,
    /// Sampler step count. Default `4` for schnell.
    pub steps: u32,
    /// Classifier-free-guidance scale. Default `1.0` for schnell.
    pub cfg: f32,
    /// KSampler `sampler_name`. Default `"euler"` for schnell.
    pub sampler_name: &'a str,
    /// KSampler `scheduler`. Default `"simple"` for schnell.
    pub scheduler: &'a str,
}

impl<'a> WorkflowSpec<'a> {
    /// Construct a FLUX.1-schnell spec with the recommended
    /// schnell-specific sampler defaults already filled in.
    ///
    /// `loras` starts empty; callers `.push(CharacterLora { .. })` on
    /// the returned value to add characters.
    pub fn flux_schnell_default(panel: PanelPrompt<'a>, output_prefix: &'a str) -> Self {
        Self {
            base_checkpoint: "flux1-schnell-fp8.safetensors",
            loras: Vec::new(),
            panel,
            output_prefix,
            steps: 4,
            cfg: 1.0,
            sampler_name: "euler",
            scheduler: "simple",
        }
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Build the ComfyUI workflow JSON from a spec-level description.
///
/// Node ordering (numeric-string keys, ascending):
///
/// | ID           | class_type              | Role                              |
/// |--------------|-------------------------|-----------------------------------|
/// | `"1"`        | `CheckpointLoaderSimple`| base model + CLIP + VAE           |
/// | `"2..2+N-1"` | `LoraLoader`            | character LoRA chain (optional)   |
/// | *last model* | — (see above)           | feeds the positive / negative CLIP|
/// | `"2+N"`      | `CLIPTextEncode`        | positive prompt                   |
/// | `"3+N"`      | `CLIPTextEncode`        | negative prompt                   |
/// | `"4+N"`      | `EmptyLatentImage`      | width / height / batch_size=1     |
/// | `"5+N"`      | `KSampler`              | seed, steps, cfg, sampler, sched. |
/// | `"6+N"`      | `VAEDecode`             | samples + VAE → image             |
/// | `"7+N"`      | `SaveImage`             | image + `filename_prefix`         |
///
/// Upstream references use `["<id>", output_index]` pairs per ComfyUI
/// convention (e.g. `"model": ["2", 0]`, `"clip": ["2", 1]`).
///
/// Byte-identical determinism: `serde_json::Map` is backed by `BTreeMap`
/// so the serialized form is stable for identical inputs. Two calls
/// with the same [`WorkflowSpec`] round-trip to the same bytes.
///
/// `@trace spec:orchestrator, spec:character-loras`
/// `@Lesson S1-500`
pub fn build_workflow(spec: &WorkflowSpec<'_>) -> Workflow {
    let mut nodes: Map<String, Value> = Map::new();

    // -- Node 1: checkpoint loader --------------------------------------
    nodes.insert(
        "1".to_string(),
        json!({
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": spec.base_checkpoint,
            }
        }),
    );

    // -- Nodes 2..(2+N-1): LoRA chain -----------------------------------
    // Each LoraLoader consumes the previous node's model+clip outputs and
    // re-exposes its own. ComfyUI's convention for LoraLoader output
    // indices is: 0 → model, 1 → clip.
    let mut prev_model_ref: Value = json!(["1", 0]);
    let mut prev_clip_ref: Value = json!(["1", 1]);
    for (i, lora) in spec.loras.iter().enumerate() {
        let node_id = (2 + i).to_string();
        nodes.insert(
            node_id.clone(),
            json!({
                "class_type": "LoraLoader",
                "inputs": {
                    "lora_name": lora.filename,
                    "strength_model": lora.model_strength,
                    "strength_clip":  lora.clip_strength,
                    "model": prev_model_ref,
                    "clip":  prev_clip_ref,
                }
            }),
        );
        prev_model_ref = json!([node_id.clone(), 0]);
        prev_clip_ref = json!([node_id, 1]);
    }

    let n = spec.loras.len();
    let id_positive = (2 + n).to_string();
    let id_negative = (3 + n).to_string();
    let id_latent = (4 + n).to_string();
    let id_sampler = (5 + n).to_string();
    let id_vaedec = (6 + n).to_string();
    let id_save = (7 + n).to_string();

    // -- Positive prompt ------------------------------------------------
    let positive_text = compose_positive(spec);
    nodes.insert(
        id_positive.clone(),
        json!({
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": positive_text,
                "clip": prev_clip_ref.clone(),
            }
        }),
    );

    // -- Negative prompt ------------------------------------------------
    // Negative shares the LoRA-chained CLIP with the positive: both prompts
    // see the same trained text-encoder modifications. Using the raw base
    // CLIP here would split conditioning across two different encoder paths
    // for no benefit — ComfyUI's stock FLUX+LoRA workflows chain both.
    nodes.insert(
        id_negative.clone(),
        json!({
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": canonicalize_prompt_text(spec.panel.negative),
                "clip": prev_clip_ref.clone(),
            }
        }),
    );

    // -- Empty latent ---------------------------------------------------
    nodes.insert(
        id_latent.clone(),
        json!({
            "class_type": "EmptyLatentImage",
            "inputs": {
                "width": spec.panel.width,
                "height": spec.panel.height,
                "batch_size": 1,
            }
        }),
    );

    // -- KSampler -------------------------------------------------------
    nodes.insert(
        id_sampler.clone(),
        json!({
            "class_type": "KSampler",
            "inputs": {
                "seed": spec.panel.seed,
                "steps": spec.steps,
                "cfg": spec.cfg,
                "sampler_name": spec.sampler_name,
                "scheduler": spec.scheduler,
                "denoise": 1.0,
                "model": prev_model_ref,
                "positive": json!([id_positive, 0]),
                "negative": json!([id_negative, 0]),
                "latent_image": json!([id_latent, 0]),
            }
        }),
    );

    // -- VAE decode -----------------------------------------------------
    nodes.insert(
        id_vaedec.clone(),
        json!({
            "class_type": "VAEDecode",
            "inputs": {
                "samples": json!([id_sampler, 0]),
                "vae": json!(["1", 2]),
            }
        }),
    );

    // -- SaveImage ------------------------------------------------------
    nodes.insert(
        id_save,
        json!({
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": spec.output_prefix,
                "images": json!([id_vaedec, 0]),
            }
        }),
    );

    // Infallible: we built a Value::Object above.
    Workflow::from_value(Value::Object(nodes))
        .expect("internal: build_workflow always produces a JSON object")
}

// ---------------------------------------------------------------------------
// Prompt composition
// ---------------------------------------------------------------------------

/// Compose the positive prompt as:
///
/// ```text
/// <trigger1> <trigger2> ... , <panel.positive> [, <panel.addendum>]
/// ```
///
/// The triggers are joined by single spaces; the separator between the
/// trigger block and the prompt body is `", "` (comma-space). The same
/// separator introduces the optional reroll addendum. Text is
/// canonicalized (NFC, CRLF→LF, outer whitespace trimmed) so
/// per-run whitespace drift cannot bust the downstream `panel_hash`.
fn compose_positive(spec: &WorkflowSpec<'_>) -> String {
    let mut out = String::new();

    // Trigger block — single spaces between triggers.
    let mut first = true;
    for lora in &spec.loras {
        let t = canonicalize_prompt_text(lora.trigger);
        if t.is_empty() {
            continue;
        }
        if !first {
            out.push(' ');
        }
        out.push_str(&t);
        first = false;
    }

    // Comma-space between the trigger block and the panel body.
    let body = canonicalize_prompt_text(spec.panel.positive);
    if !out.is_empty() && !body.is_empty() {
        out.push_str(", ");
    }
    out.push_str(&body);

    // Optional reroll addendum, same comma-space separator.
    if let Some(add) = spec.panel.addendum {
        let add = canonicalize_prompt_text(add);
        if !add.is_empty() {
            if !out.is_empty() {
                out.push_str(", ");
            }
            out.push_str(&add);
        }
    }

    out
}

/// Canonicalize a prompt string the same way hash-input canonicalization
/// does — NFC, LF line endings, trimmed outer whitespace. We *don't*
/// collapse internal blank-line runs here because prompts are flat text
/// and the hash layer lives upstream; this is the conservative NFC+trim
/// suitable for prompt weaving.
fn canonicalize_prompt_text(s: &str) -> String {
    let nfc: String = s.nfc().collect();
    let lf = nfc.replace("\r\n", "\n").replace('\r', "\n");
    lf.trim().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod workflow_tests {
    use super::*;

    fn sample_panel<'a>(pos: &'a str, neg: &'a str, seed: u64) -> PanelPrompt<'a> {
        PanelPrompt {
            positive: pos,
            negative: neg,
            addendum: None,
            width: 1024,
            height: 1024,
            seed,
        }
    }

    fn spec_no_loras<'a>(panel: PanelPrompt<'a>) -> WorkflowSpec<'a> {
        WorkflowSpec::flux_schnell_default(panel, "tt-01-p1")
    }

    fn class_types(wf: &Workflow) -> Vec<(String, String)> {
        let obj = wf.as_json().as_object().expect("object");
        let mut v: Vec<(String, String)> = obj
            .iter()
            .map(|(k, v)| {
                let ct = v
                    .get("class_type")
                    .and_then(|c| c.as_str())
                    .unwrap_or_default()
                    .to_string();
                (k.clone(), ct)
            })
            .collect();
        // Sort by numeric ID so assertions are independent of the
        // underlying Map's iteration order.
        v.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(u32::MAX));
        v
    }

    #[test]
    fn zero_loras_produces_seven_nodes() {
        let panel = sample_panel("a scene", "bad art", 42);
        let wf = build_workflow(&spec_no_loras(panel));
        let cts = class_types(&wf);
        // Zero LoRAs → exactly seven nodes: ckpt, +clip, -clip, latent,
        // sampler, vae-decode, save. The `class_type` list below also
        // falsifies silent renumbering regressions.
        assert_eq!(cts.len(), 7, "expected seven nodes, got {cts:?}");
        let expected = [
            ("1", "CheckpointLoaderSimple"),
            ("2", "CLIPTextEncode"),
            ("3", "CLIPTextEncode"),
            ("4", "EmptyLatentImage"),
            ("5", "KSampler"),
            ("6", "VAEDecode"),
            ("7", "SaveImage"),
        ];
        for (id, ct) in expected {
            let got = cts.iter().find(|(k, _)| k == id);
            assert_eq!(
                got.map(|(_, c)| c.as_str()),
                Some(ct),
                "node {id} class_type",
            );
        }
    }

    #[test]
    fn one_lora_produces_eight_nodes_and_prepends_trigger() {
        let panel = sample_panel("a scene", "bad art", 42);
        let mut spec = spec_no_loras(panel);
        spec.loras.push(CharacterLora {
            filename: "tlatoani-v1.safetensors",
            trigger: "TlhAxolotlSage",
            model_strength: 1.0,
            clip_strength: 1.0,
        });
        let wf = build_workflow(&spec);
        let cts = class_types(&wf);
        assert_eq!(cts.len(), 8, "1 LoRA → 8 nodes");

        // Node 2 is the LoraLoader.
        assert_eq!(
            cts.iter().find(|(k, _)| k == "2").map(|(_, c)| c.as_str()),
            Some("LoraLoader"),
        );
        // Node 3 is the positive CLIPTextEncode (first after the single LoRA).
        let pos_node = wf
            .as_json()
            .get("3")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert!(
            pos_node.starts_with("TlhAxolotlSage, "),
            "positive should prepend trigger token, got {pos_node:?}",
        );
    }

    #[test]
    fn two_loras_chain_models_and_concatenate_triggers() {
        let panel = sample_panel("a scene", "bad art", 1);
        let mut spec = spec_no_loras(panel);
        spec.loras.push(CharacterLora {
            filename: "tlatoani-v1.safetensors",
            trigger: "TlhAxolotlSage",
            model_strength: 1.0,
            clip_strength: 1.0,
        });
        spec.loras.push(CharacterLora {
            filename: "covi-v1.safetensors",
            trigger: "CoviFigure",
            model_strength: 1.0,
            clip_strength: 1.0,
        });
        let wf = build_workflow(&spec);
        assert_eq!(class_types(&wf).len(), 9, "2 LoRAs → 9 nodes");

        // Trigger block reads "TlhAxolotlSage CoviFigure, a scene".
        let positive = wf
            .as_json()
            .get("4")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert_eq!(positive, "TlhAxolotlSage CoviFigure, a scene");

        // Second LoRA (node 3) consumes node 2's outputs.
        let chain = wf
            .as_json()
            .get("3")
            .and_then(|n| n.get("inputs"))
            .unwrap();
        assert_eq!(chain.get("model").unwrap(), &json!(["2", 0]));
        assert_eq!(chain.get("clip").unwrap(), &json!(["2", 1]));
    }

    #[test]
    fn seed_lands_exactly_in_ksampler() {
        let panel = sample_panel("x", "y", 314159);
        let wf = build_workflow(&spec_no_loras(panel));
        // With 0 LoRAs, KSampler is node 5.
        let seed = wf
            .as_json()
            .get("5")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("seed"))
            .and_then(|s| s.as_u64())
            .unwrap();
        assert_eq!(seed, 314_159);
    }

    #[test]
    fn addendum_appends_with_leading_comma_space() {
        let mut panel = sample_panel("a scene", "bad art", 1);
        panel.addendum = Some("avoid: double tail");
        let wf = build_workflow(&spec_no_loras(panel));
        // No LoRAs, so positive CLIPTextEncode is node 2.
        let positive = wf
            .as_json()
            .get("2")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert_eq!(positive, "a scene, avoid: double tail");
    }

    #[test]
    fn addendum_none_leaves_positive_untouched() {
        let panel = sample_panel("a scene", "bad art", 1);
        let wf = build_workflow(&spec_no_loras(panel));
        let positive = wf
            .as_json()
            .get("2")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert_eq!(positive, "a scene");
    }

    #[test]
    fn build_workflow_is_byte_deterministic() {
        let panel_a = sample_panel("a scene", "bad art", 42);
        let panel_b = sample_panel("a scene", "bad art", 42);
        let a = build_workflow(&spec_no_loras(panel_a));
        let b = build_workflow(&spec_no_loras(panel_b));
        let sa = serde_json::to_string(a.as_json()).unwrap();
        let sb = serde_json::to_string(b.as_json()).unwrap();
        assert_eq!(sa, sb, "identical spec must produce byte-identical JSON");
    }

    #[test]
    fn different_seeds_produce_different_json() {
        let a = build_workflow(&spec_no_loras(sample_panel("p", "n", 1)));
        let b = build_workflow(&spec_no_loras(sample_panel("p", "n", 2)));
        let sa = serde_json::to_string(a.as_json()).unwrap();
        let sb = serde_json::to_string(b.as_json()).unwrap();
        assert_ne!(sa, sb, "seed drift must show up in the JSON");
    }

    #[test]
    fn output_prefix_lands_in_save_image() {
        let panel = sample_panel("p", "n", 1);
        let mut spec = WorkflowSpec::flux_schnell_default(panel, "tt-07-p2");
        spec.loras.clear();
        let wf = build_workflow(&spec);
        // With 0 LoRAs, SaveImage is node 7.
        let prefix = wf
            .as_json()
            .get("7")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("filename_prefix"))
            .and_then(|p| p.as_str())
            .unwrap();
        assert_eq!(prefix, "tt-07-p2");
    }

    #[test]
    fn positive_and_negative_do_not_leak_across_nodes() {
        let panel = sample_panel(
            "warm paper scene with Covi",
            "photo-realism, double tail, 3D render",
            99,
        );
        let wf = build_workflow(&spec_no_loras(panel));
        // Positive = node 2, negative = node 3 (0 LoRAs).
        let pos = wf
            .as_json()
            .get("2")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        let neg = wf
            .as_json()
            .get("3")
            .and_then(|n| n.get("inputs"))
            .and_then(|i| i.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert_eq!(pos, "warm paper scene with Covi");
        assert_eq!(neg, "photo-realism, double tail, 3D render");
        assert!(!pos.contains("photo-realism"));
        assert!(!neg.contains("warm paper"));
    }

    #[test]
    fn schnell_defaults_are_wired_into_ksampler() {
        let panel = sample_panel("p", "n", 42);
        let spec = spec_no_loras(panel);
        let wf = build_workflow(&spec);
        let inputs = wf.as_json().get("5").and_then(|n| n.get("inputs")).unwrap();
        assert_eq!(inputs.get("steps").unwrap(), &json!(4));
        assert_eq!(inputs.get("cfg").unwrap(), &json!(1.0));
        assert_eq!(inputs.get("sampler_name").unwrap(), &json!("euler"));
        assert_eq!(inputs.get("scheduler").unwrap(), &json!("simple"));
    }

    #[test]
    fn canonicalizes_prompt_nfc_and_crlf() {
        // Decomposed Tlatoāni + CRLF — both must wash out to the same text
        // as the precomposed, LF version.
        let decomposed = "Tlatoa\u{0304}ni warm paper\r\n";
        let composed = "Tlatoāni warm paper";
        let a = build_workflow(&spec_no_loras(sample_panel(decomposed, "neg", 1)));
        let b = build_workflow(&spec_no_loras(sample_panel(composed, "neg", 1)));
        let sa = serde_json::to_string(a.as_json()).unwrap();
        let sb = serde_json::to_string(b.as_json()).unwrap();
        assert_eq!(sa, sb, "NFC + LF canonicalization must align JSON");
    }
}
