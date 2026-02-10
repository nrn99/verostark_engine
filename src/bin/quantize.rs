use candle_core::{Device, Tensor, DType};
use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: quantize <input.safetensors> <output.safetensors>");
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    println!("Reading tensors from {}", input_path);
    let device = Device::Cpu;
    let tensors = candle_core::safetensors::load(input_path, &device)?;

    let mut new_tensors: HashMap<String, Tensor> = HashMap::new();

    for (name, tensor) in tensors {
        // Convert to F16 for 50% size reduction
        // Keep embeddings/classifiers in F32 if needed? Usually F16 is fine for inference
        // but candle-transformers BERT might expect specific types or handle casting.
        // Let's safe-cast everything to F16.
        let new_tensor = tensor.to_dtype(DType::F16)?;
        new_tensors.insert(name, new_tensor);
    }

    println!("Saving F16 tensors to {}", output_path);
    candle_core::safetensors::save(&new_tensors, output_path)?;
    println!("Quantization Complete.");
    Ok(())
}
