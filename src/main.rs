mod math;
use bip39;
use math::field;
use math::lagrange;
use rand_core::{OsRng, RngCore};
use std::{error::Error, io};
use structopt::StructOpt;

#[derive(Debug)]
struct MainError(String);

impl std::fmt::Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MainError {}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "seed-split",
    about = "Split a seed phrase into multiple shares."
)]
enum Opt {
    /// Generate a random seed phrase.
    Random,
    /// Split a seed phrase into multiple shares.
    Split {
        /// The number of shares needed to recreate the seed.
        #[structopt(short = "t", long = "threshold")]
        threshold: u8,
        /// The total number of shares.
        #[structopt(short = "n", long = "count")]
        count: u8,
    },
    /// Combine multiple shares into a seed phrase.
    Combine {
        /// The number of shares being combined
        #[structopt(name = "threshold")]
        threshold: u8,
    },
}

fn random() -> Result<(), Box<dyn Error>> {
    let mut entropy_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut entropy_bytes);
    let seed_phrase = bip39::Mnemonic::from_entropy(&entropy_bytes)
        .expect("failed to generate mnemonic from entropy");
    println!("{}", seed_phrase);
    Ok(())
}

fn continue_split_128(data: [u8; 16], sharing: lagrange::Sharing) -> Result<(), Box<dyn Error>> {
    let secret = field::GF128::from(data);
    let shares = lagrange::split(&mut OsRng, secret, sharing);
    for (i, share) in shares {
        let share_bytes: [u8; 16] = share.into();
        let mnemonic = bip39::Mnemonic::from_entropy(&share_bytes)?;
        println!("{} {}", u8::from(i) + 1, mnemonic);
    }
    Ok(())
}

fn continue_split_256(data: [u8; 32], sharing: lagrange::Sharing) -> Result<(), Box<dyn Error>> {
    let secret = field::GF256::from(data);
    let shares = lagrange::split(&mut OsRng, secret, sharing);
    for (i, share) in shares {
        let share_bytes: [u8; 32] = share.into();
        let mnemonic = bip39::Mnemonic::from_entropy(&share_bytes)?;
        println!("{} {}", u8::from(i) + 1, mnemonic);
    }
    Ok(())
}

fn split(threshold: u8, count: u8) -> Result<(), Box<dyn Error>> {
    if count <= 0 {
        return Err(Box::new(MainError(format!("count must be at least 1"))));
    }
    if threshold > count {
        return Err(Box::new(MainError(format!("threshold must be <= count"))));
    }
    let sharing = lagrange::Sharing::new(threshold, count);
    println!("Seed Phrase:");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let mnemonic = bip39::Mnemonic::parse(&buf)?;
    let entropy = mnemonic.to_entropy();
    if entropy.len() <= 16 {
        let mut data = [0u8; 16];
        data[..entropy.len()].copy_from_slice(&entropy);
        continue_split_128(data, sharing)
    } else if entropy.len() <= 32 {
        let mut data = [0u8; 32];
        data.copy_from_slice(&entropy);
        continue_split_256(data, sharing)
    } else {
        Err(Box::new(MainError(format!(
            "excessive seed length: {} bytes",
            entropy.len()
        ))))
    }
}

fn combine(threshold: u8) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn main() {
    let res = match Opt::from_args() {
        Opt::Random => random(),
        Opt::Split { threshold, count } => split(threshold, count),
        Opt::Combine { threshold } => combine(threshold),
    };
    if let Err(e) = res {
        println!("error: {}", e);
    }
}
