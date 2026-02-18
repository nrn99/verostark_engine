use candle_core::{Device, Tensor};
use candle_nn::{VarBuilder, Module, Linear, linear};
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;
use std::path::Path;
use std::collections::HashMap;

use crate::scrubber::ScrubResult;


use log::{info, error};

// Manual implementation since candle-transformers doesn't expose it
struct BertForTokenClassification {
    bert: BertModel,
    classifier: Linear,
}

impl BertForTokenClassification {
    fn load(vb: VarBuilder, config: &Config, num_labels: usize) -> Result<Self, candle_core::Error> {
        let bert = BertModel::load(vb.clone(), config)?;
        let hidden_size = config.hidden_size;
        let classifier = linear(hidden_size, num_labels, vb.pp("classifier"))?;
        Ok(Self { bert, classifier })
    }

    fn forward(&self, input_ids: &Tensor, token_type_ids: &Tensor, attention_mask: Option<&Tensor>) -> Result<Tensor, candle_core::Error> {
        let hidden_states = self.bert.forward(input_ids, token_type_ids, attention_mask)?;
        self.classifier.forward(&hidden_states)
    }
}

pub struct SmartScrubber {
    model: BertForTokenClassification,
    tokenizer: Tokenizer,
    device: Device,
    // Label mapping: 0 -> O, 1 -> B-MISC, 2 -> I-MISC, 3 -> B-PER, 4 -> I-PER, ...
    // Standard BERT-NER labels usually:
    // O, B-MISC, I-MISC, B-PER, I-PER, B-ORG, I-ORG, B-LOC, I-LOC
    id2label: Vec<String>,
}

impl SmartScrubber {
    pub fn new(model_path: &str, tokenizer_path: &str, config_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let device = Device::Cpu;
        
        // Load Tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| e.to_string())?;
        
        // Load Config
        let config_str = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;
        
        // Load Model Weights (Read F16 file, cast to F32 for CPU stability)
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], candle_core::DType::F32, &device)? };
        
        // Standard BERT NER labels (CoNLL-2003)
        let id2label = vec![
            "O".to_string(),
            "B-MISC".to_string(), "I-MISC".to_string(),
            "B-PER".to_string(), "I-PER".to_string(),
            "B-ORG".to_string(), "I-ORG".to_string(),
            "B-LOC".to_string(), "I-LOC".to_string(),
        ];
        
        let model = BertForTokenClassification::load(vb, &config, id2label.len())?;

        info!("Verostark Cortex v1.1 (Phantom+Broad) Loaded Successfully on CPU.");
        Ok(Self {
            model,
            tokenizer,
            device,
            id2label,
        })
    }

    pub fn scrub(&self, text: &str) -> ScrubResult {
        if text.trim().is_empty() {
            return ScrubResult {
                text: text.to_string(),
                total: 0,
                details: HashMap::new(),
            };
        }

        // Physics Hack: Phantom Context
        // If text is too short (< 8 words), BERT fails to see context.
        // We wrap it: "Hello, regarding [TEXT] today."
        // Then we extract the scrubbed part.
        let word_count = text.split_whitespace().count();
        let use_phantom = word_count < 8;
        
        let process_text = if use_phantom {
            format!("Hello, regarding {} today.", text)
        } else {
            text.to_string()
        };

        let result = self.scrub_internal(&process_text);

        if use_phantom {
            // Unwrap: "Hello, regarding [SCRUBBED] today."
            // Prefix len: "Hello, regarding ".len() = 17
            // Suffix len: " today.".len() = 7
            let prefix_len = 17;
            let suffix_len = 7;
            if result.text.len() > prefix_len + suffix_len {
                let real_scrubbed = &result.text[prefix_len..result.text.len()-suffix_len];
                return ScrubResult {
                    text: real_scrubbed.to_string(),
                    total: result.total,
                    details: result.details,
                };
            }
            // Fallback if length mismatch (shouldn't happen unless aggressive scrubbing changed structure)
            return ScrubResult {
                text: text.to_string(),
                total: 0,
                details: HashMap::new(),
            };
        }

        result
    }

    fn scrub_internal(&self, text: &str) -> ScrubResult {
         let mut empty_res = ScrubResult {
            text: text.to_string(),
            total: 0,
            details: HashMap::new(),
        };

        if text.trim().is_empty() {
            return empty_res;
        }

        // Tokenize
        // Note: Input limit is 512 tokens for BERT.
        let encoding = match self.tokenizer.encode(text, true) {
            Ok(e) => e,
            Err(e) => {
                error!("Tokenizer failed: {}", e);
                return empty_res;
            }
        };

        let _tokens = encoding.get_tokens();
        let token_ids = encoding.get_ids();
        // Inference
        let input_ids = match Tensor::new(token_ids, &self.device).and_then(|t| t.unsqueeze(0)) {
            Ok(t) => t,
            Err(_) => return empty_res,
        };
        
        let token_type_ids = match input_ids.zeros_like() {
            Ok(t) => t,
            Err(_) => return empty_res,
        };

        let output = match self.model.forward(&input_ids, &token_type_ids, None) {
            Ok(o) => o,
            Err(_) => return empty_res,
        };

        // Get logits (batch, seq_len, num_labels)
        let logits = output.squeeze(0).unwrap();
        
        // Simple Argmax decoding
        let mut new_text = String::new();
        let mut scrub_count = 0;
        let mut details = HashMap::new();
        
        // Reconstruct string logic is complex with subwords. 
        // For MVP, if we detect entities, we can just replace the whole text or try to map back to spans.
        // Mapping tokens back to char spans is supported by Tokenizer.
        
        // Let's do a scan:
        // iterate tokens, find spans of PER/LOC.
        
        let (_dim_seq, _dim_labels) = logits.dims2().unwrap();
        let best_labels: Vec<u32> = logits.argmax(1).unwrap().to_vec1().unwrap();
        
        // 0=O, 3=B-PER, 4=I-PER, 7=B-LOC, 8=I-LOC
        
        // We accumulate spans to mask.
        let mut mask_spans: Vec<(usize, usize, &str)> = Vec::new(); // start, end, label
        
        let mut current_entity_start: Option<usize> = None;
        let mut current_label = "";
        
        for (i, &label_idx) in best_labels.iter().enumerate() {
            let label_str = self.id2label.get(label_idx as usize).map(|s| s.as_str()).unwrap_or("O");
            
            // Check output logic
            // B-XXX starts entity. I-XXX continues. O ends.
            
            // Tuning: Broad Spectrum Filter (Detect PER, ORG, LOC)
            if label_str.starts_with("B-") {
                // End previous if exists
                if let Some(start) = current_entity_start {
                   if let Some(offsets) = encoding.get_offsets().get(i-1) {
                       mask_spans.push((start, offsets.1, current_label));
                   }
                }
                // Start new
                if let Some(offsets) = encoding.get_offsets().get(i) {
                     current_entity_start = Some(offsets.0);
                     // Map labels to verified tags
                     current_label = if label_str.contains("PER") { "<PERSON>" }
                                     else if label_str.contains("LOC") { "<LOCATION>" }
                                     else if label_str.contains("ORG") { "<ORG>" }
                                     else { "" };
                     
                     if current_label.is_empty() { current_entity_start = None; }
                }
            } else if label_str.starts_with("I-") {
                // Continue if matching logic could be smarter, but simplistic "continue if started" works for now.
            } else {
                // O or match end
                if let Some(start) = current_entity_start {
                     if let Some(offsets) = encoding.get_offsets().get(i-1) {
                         // End the entity
                         mask_spans.push((start, offsets.1, current_label));
                     }
                     current_entity_start = None;
                }
            }
        }
        
        // Reconstruction loop could be simpler:
        // Use the original text and offsets to build a new string.
        if mask_spans.is_empty() {
             return empty_res;
        }

        // Apply replacements (reverse order to keep indices valid? Or build from left to right)
        // Building L->R is safer if we just verify overlaps.
        // Assuming tokenizer offsets are sorted.
        
        let mut last_pos = 0;
        for (start, end, label) in mask_spans {
            // Safety check
            if start >= last_pos && end <= text.len() {
                new_text.push_str(&text[last_pos..start]);
                new_text.push_str(label);
                last_pos = end;
                scrub_count += 1;
                
                // Track details
                *details.entry(label.to_string()).or_insert(0) += 1;
            }
        }
        new_text.push_str(&text[last_pos..]);

        ScrubResult {
            text: new_text,
            total: scrub_count,
            details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_scrubber_inference() {
        // Needs model files present in model/
        if !Path::new("model/model.safetensors").exists() {
            eprintln!("SKIPPING TEST: Model files not found. Run download_model.sh first.");
            return;
        }

        let scrubber = SmartScrubber::new(
            "model/model.safetensors",
            "model/tokenizer.json",
            "model/config.json"
        ).expect("Failed to load model");

        // Test Case 1: Person Name
        let input = "My name is John Smith.";
        let res = scrubber.scrub(input);
        println!("Input: {}, Scrubbed: {}", input, res.text);
        assert!(res.total >= 1);
        assert!(res.text.contains("<PERSON>"));

        // Test Case 2: Location
        let input = "I live in Paris.";
        let res = scrubber.scrub(input);
        println!("Input: {}, Scrubbed: {}", input, res.text);
        assert!(res.total >= 1);
        assert!(res.text.contains("<LOCATION>"));
    }
}
