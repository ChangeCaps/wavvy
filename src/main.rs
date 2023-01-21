use std::{error::Error, fs::File, io::prelude::*, path::PathBuf};

use clap::Parser;
use wav::BitDepth;

#[derive(Parser)]
struct Args {
    path: PathBuf,
    #[clap(short, long, default_value = "1")]
    downsample: u32,
    #[clap(short, long, default_value = "1.0")]
    gain: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut file = File::open(args.path)?;
    let (header, data) = wav::read(&mut file)?;
    let samples = convert_samples(data);

    let mut out = File::create("out.h")?;

    writeln!(out, "#include <stdint.h>")?;

    writeln!(
        out,
        "#define SAMPLE_RATE {}",
        header.sampling_rate / args.downsample
    )?;
    writeln!(out, "#define CHANNEL_COUNT {}", header.channel_count)?;
    writeln!(out, "#define BYTES_PER_SECOND {}", header.bytes_per_second)?;
    writeln!(out, "#define BYTES_PER_SAMPLE {}", header.bytes_per_sample)?;
    writeln!(out, "#define BITS_PER_SAMPLE {}", header.bits_per_sample)?;
    writeln!(
        out,
        "#define SAMPLES_COUNT {}",
        samples.len() / args.downsample as usize
    )?;
    writeln!(out, "const uint8_t samples[] PROGMEM = {{")?;

    for sample in samples.chunks(args.downsample as usize * header.channel_count as usize) {
        let sample = apply_gain(sample[0], args.gain);

        writeln!(out, "    {},", sample)?;
    }

    writeln!(out, "}};")?;

    println!("Output written to out.h");

    Ok(())
}

fn apply_gain(sample: u8, gain: f32) -> u8 {
    let sample = sample as f32 - 128.0;
    let sample = sample * gain;
    (sample + 128.0) as u8
}

fn convert_samples(samples: BitDepth) -> Vec<u8> {
    match samples {
        BitDepth::Eight(samples) => samples,
        BitDepth::Sixteen(samples) => samples
            .into_iter()
            .map(|sample| (((sample as i32 + 32768) >> 8) & 0xFF) as u8)
            .collect(),
        BitDepth::TwentyFour(_) => todo!(),
        BitDepth::ThirtyTwoFloat(_) => todo!(),
        BitDepth::Empty => todo!(),
    }
}
