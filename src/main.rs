mod math;
use structopt::StructOpt;

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

fn random() {}

fn split(treshold: u8, count: u8) {}

fn combine(threshold: u8) {}

fn main() {
    match Opt::from_args() {
        Opt::Random => random(),
        Opt::Split { threshold, count } => split(threshold, count),
        Opt::Combine { threshold } => combine(threshold),
    }
}
