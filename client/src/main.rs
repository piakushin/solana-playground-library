mod args;

use std::{path::Path, thread::sleep, time::Duration};

use anyhow::{anyhow, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::{self, write_keypair_file, Keypair},
    signer::Signer,
    system_transaction,
    transaction::Transaction,
};

use crate::args::{Action, Args};

const LOCAL_VALIDATOR: &str = "http://localhost:8899";
const FIRST_KEYPATH: &str = "./dist/first.json";
const SECOND_KEYPATH: &str = "./dist/second.json";

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

const TRANSFER_LAMPORTS_PROGRAM_ID: &str = "H3mNtWPT1MFbNkb5J4JHmimzWhp4oNB7KawhaPrZsBtR";
const SYSTEM_ACCOUNT_ID: &str = "11111111111111111111111111111111";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InstructionData {
    pub vault_bump_seed: u8,
    pub transfer_amount: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let client = RpcClient::new(LOCAL_VALIDATOR);
    match args.action {
        Some(Action::GenKeyPair { outfile }) => gen_keypair(outfile),
        Some(Action::AirDrop { keypair_file, sol }) => {
            let keypair = read_keypair_file(keypair_file)?;
            airdrop(&client, &keypair.pubkey(), sol_to_lamports(sol))
        }
        Some(Action::CheckBalance { keypair_file }) => {
            let keypair = read_keypair_file(keypair_file)?;
            print_balance(&client, &keypair.pubkey())
        }
        Some(Action::Transfer {
            from_keypair_file,
            to_keypair_file,
            sol,
        }) => {
            let from_keypair = read_keypair_file(from_keypair_file)?;
            let to_keypair = read_keypair_file(to_keypair_file)?;

            let to_pubkey = to_keypair.pubkey();
            let lamports = sol_to_lamports(sol);
            transfer_funds(&client, &from_keypair, &to_pubkey, lamports)
        }
        Some(Action::CustomTransfer {
            to_keypair_file,
            sol,
        }) => {
            let first = read_keypair_file(FIRST_KEYPATH)?;

            let program_id = TRANSFER_LAMPORTS_PROGRAM_ID.parse()?;
            let (second, vault_bump_seed) =
                Pubkey::find_program_address(&[b"recipient", first.pubkey().as_ref()], &program_id);
            println!("found program address: {}", second);

            let accounts = vec![
                AccountMeta::new(first.pubkey(), true),
                AccountMeta::new(second, false),
                AccountMeta::new(SYSTEM_ACCOUNT_ID.parse()?, false),
            ];

            let transfer_amount = sol_to_lamports(sol);
            let instruction_data = InstructionData {
                vault_bump_seed,
                transfer_amount,
            };
            let instruction = Instruction::new_with_borsh(program_id, &instruction_data, accounts);

            let message = Message::new(&[instruction], Some(&first.pubkey()));
            let recent_blockhash = client.get_latest_blockhash()?;

            let transaction = Transaction::new(&[&first], message, recent_blockhash);

            client.send_and_confirm_transaction_with_spinner(&transaction)?;

            Ok(())
        }
        None => Ok(()),
    }
}

fn airdrop(client: &RpcClient, pubkey: &Pubkey, lamports: u64) -> Result<()> {
    let signature = client.request_airdrop(pubkey, lamports)?;
    while !client.confirm_transaction(&signature)? {
        println!("Waiting for confirmation . . . ");
        sleep(Duration::from_millis(1000));
    }
    println!("Confirmed airdrop {lamports} lamports to {pubkey}");
    Ok(())
}

fn gen_keypair(outfile: impl AsRef<Path>) -> Result<()> {
    let keypair = Keypair::new();
    write_keypair_file(&keypair, &outfile).map_err(|e| anyhow!("write keypair failed: {e}"))?;
    println!("keys generated to: {}", outfile.as_ref().display());
    Ok(())
}

fn print_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<()> {
    let balance = client.get_balance(pubkey)?;
    let mut account = pubkey.to_string();
    account.truncate(5);
    println!(
        "{} balance: {}",
        account,
        balance as f64 / LAMPORTS_PER_SOL as f64
    );
    Ok(())
}

fn transfer_funds(
    client: &RpcClient,
    from_keypair: &Keypair,
    to_pubkey: &Pubkey,
    lamports: u64,
) -> Result<()> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction =
        system_transaction::transfer(from_keypair, to_pubkey, lamports, recent_blockhash);
    let res = client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .unwrap();
    println!("signature: {res}");

    Ok(())
}

fn read_keypair_file(path: impl AsRef<Path>) -> Result<Keypair> {
    signature::read_keypair_file(path).map_err(|e| anyhow!("read keypair failed: {e}"))
}

fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL as f64) as u64
}
