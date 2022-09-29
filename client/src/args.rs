use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand)]
pub enum Action {
    GenKeyPair {
        outfile: String,
    },
    AirDrop {
        keypair_file: String,
        sol: f64,
    },
    CheckBalance {
        keypair_file: String,
    },
    Transfer {
        from_keypair_file: String,
        to_keypair_file: String,
        sol: f64,
    },
    CustomTransfer {
        to_keypair_file: String,
        sol: f64,
    },
}
