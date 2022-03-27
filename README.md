# seed-split

A tool to split seed phrases into shares.

For example, you can generate a seed phrase:

```
$ seed-split random
genre cradle verb jazz super pizza silver limit hungry grace choose sing
```

Then, you can split that seed phrase into 3 shares:

```
$ seed-split split -t 2 -n 3
Seed Phrase:
genre cradle verb jazz super pizza silver limit hungry grace choose sing
1 father day shaft path tilt festival loud crystal fence fan immune section
2 liquid captain owner such put during festival silly bracket acquire oppose target
3 rail cloth portion avocado nice regret border lab calm culture second stock
```

Since we've used a threshold of 2, any 2 of these shares are enough to
reconstruct the seed phrase:

```
$ seed-split combine 2
1 father day shaft path tilt festival loud crystal fence fan immune section
3 rail cloth portion avocado nice regret border lab calm culture second stock
Reconstructed:
genre cradle verb jazz super pizza silver limit hungry grace choose sing
```

The advantage of this approach is that you can store the seed phrase over
different media, or even with different people. For example,
continuing with threshold of 2, you could
give one share to a friend, which isn't enough to reconstruct the seed phrase
by itself, but would serve as a backup if you were to lose one of the other
two shares.

# Security

This is experimental software, use at your own risk.

# Usage

```
seed-split 0.1.0
Split a seed phrase into multiple shares.

USAGE:
    seed-split <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    combine    Combine multiple shares into a seed phrase
    help       Prints this message or the help of the given subcommand(s)
    random     Generate a random seed phrase
    split      Split a seed phrase into multiple shares
```

## Generating Seed Phrases

```
seed-split-random 0.1.0
Generate a random seed phrase

USAGE:
    seed-split random [FLAGS]

FLAGS:
    -h, --help       Prints help information
        --long       If set, generate a 256 bit seed phrase instead
    -V, --version    Prints version information
```

## Splitting Seed Phrases

```
seed-split-split 0.1.0
Split a seed phrase into multiple shares

USAGE:
    seed-split split --count <count> --threshold <threshold>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --count <count>            The total number of shares
    -t, --threshold <threshold>    The number of shares needed to recreate the seed
```

The program will then ask for you to enter the seed phrase on the command line.

## Combining Shares

```
seed-split-combine 0.1.0
Combine multiple shares into a seed phrase

USAGE:
    seed-split combine <threshold>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <threshold>    The number of shares being combined
```

The program will then ask you for `<threshold>` shares on the command line.

# Implementation

The idea is to use [Shamir's Secret Sharing](https://www.wikiwand.com/en/Shamir%27s_Secret_Sharing)
over either `GF(2^128)` or `GF(2^256)`, depending on whether or not we're
using 128 bit or 256 bit seed phrases. Not much else is going on really.
The only itch I had to scratch was to encode the shares as seed phrases once
again, instead of hex strings like other tools do.
