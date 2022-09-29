use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint as entrypoint_macro,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction::create_account,
};

entrypoint_macro!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InstructionData {
    pub vault_bump_seed: u8,
    pub transfer_amount: u64,
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    mut instruction_data: &[u8],
) -> ProgramResult {
    msg!("program id: {}", program_id);
    let instruction_data = InstructionData::deserialize(&mut instruction_data)?;
    msg!("instruction data: {:#?}", instruction_data);

    let accounts_iter = &mut accounts.iter();

    let dst = next_account_info(accounts_iter)?;
    let src = next_account_info(accounts_iter)?;
    msg!("all accounts are present");

    let instruction = create_account(dst.key, src.key, 1_000_000, 1024, program_id);

    let account_infos = &[dst.clone(), src.clone()];

    let bump_seed = instruction_data.vault_bump_seed;
    invoke_signed(
        &instruction,
        account_infos,
        &[&[b"recipient", dst.key.as_ref(), &[bump_seed]]],
    )?;
    msg!("create account instruction invoked");

    msg!(
        "old balance: src: {}, dest: {}",
        dst.lamports(),
        src.lamports()
    );

    **dst.try_borrow_mut_lamports()? += instruction_data.transfer_amount;

    **src.try_borrow_mut_lamports()? -= instruction_data.transfer_amount;

    msg!(
        "transfered {} from {} to {}",
        instruction_data.transfer_amount,
        dst.key,
        src.key
    );
    msg!(
        "new balance: src: {}, dest: {}",
        dst.lamports(),
        src.lamports()
    );

    Ok(())
}
