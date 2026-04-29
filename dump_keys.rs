use candle_core::Device;
use candle_core::safetensors::load;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let home = std::env::var("HOME")?;
    let path = PathBuf::from(home).join(".local/share/bloatshot/model.safetensors");
    let device = Device::Cpu;
    let tensors = candle_core::safetensors::load(&path, &device)?;
    
    println!("Found {} tensors.", tensors.len());
    let mut keys: Vec<_> = tensors.keys().collect();
    keys.sort();
    for key in keys.iter().take(50) {
        println!("{}", key);
    }
    Ok(())
}
