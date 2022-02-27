//! Processor asks instruction.rs to decode the instruction_data argument from the entrypoint function.
//! Using the decoded data, processor decides which processing function to use to process the request.
//! The processor may use state.rs to encode/decode the state of an account which has been passed into the entrypoint.

use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::StreamError,
    instruction::StreamInstruction,
    state::{CreateStreamInput, StreamData, WithdrawInput},
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = StreamInstruction::unpack(instruction_data)?;

        match instruction {
            StreamInstruction::CreateStream(data) => {
                Self::process_create_stream(program_id, accounts, data)
            }
            StreamInstruction::WithdrawFromStream(data) => {
                Self::process_withdraw(program_id, accounts, data)
            }
            StreamInstruction::CloseStream => Self::process_close(program_id, accounts),
        }
    }

    fn process_create_stream(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: CreateStreamInput,
    ) -> ProgramResult {
        let admin_pub_key = match Pubkey::from_str("ADMIN_PUBLIC_KEY_HERE") {
            Ok(key) => key,
            Err(_) => return Err(StreamError::PubKeyParseError.into()),
        };

        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let admin_account = next_account_info(account_info_iter)?;

        if *admin_account.key != admin_pub_key {
            return Err(StreamError::AdminAccountInvalid.into());
        }

        // 0.03 sol token admin acccount fee
        // 30000000 Lamports = 0.03 sol
        **escrow_account.try_borrow_mut_lamports()? -= 30000000;
        **admin_account.try_borrow_mut_lamports()? += 30000000;

        if data.end_time <= data.start_time || data.start_time < Clock::get()?.unix_timestamp {
            return Err(StreamError::InvalidStartOrEndTime.into());
        }

        if data.amount_second * ((data.end_time - data.start_time) as u64)
            != **escrow_account.lamports.borrow()
                - Rent::get()?.minimum_balance(escrow_account.data_len())
        {
            return Err(StreamError::NotEnoughLamports.into());
        }

        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *receiver_account.key != data.receiver {
            return Err(ProgramError::InvalidAccountData);
        }

        // We will first create escrow_data and then
        // with the help of borsh serialize method we can write data to escrow_account
        let escrow_data = StreamData::new(data, *sender_account.key);
        escrow_data.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_withdraw(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: WithdrawInput,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;

        let mut escrow_data = StreamData::try_from_slice(&escrow_account.data.borrow())
            .expect("failed to serialize escrow data");

        if *receiver_account.key != escrow_data.receiver {
            return Err(ProgramError::IllegalOwner);
        }

        if !receiver_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let time = Clock::get()?.unix_timestamp;

        let total_token_owned = escrow_data.amount_second
            * ((std::cmp::min(time, escrow_data.end_time) - escrow_data.start_time) as u64)
            - escrow_data.lamports_withdrawn;

        if data.amount > total_token_owned {
            return Err(StreamError::WithdrawError.into());
        }

        **escrow_account.try_borrow_mut_lamports()? -= data.amount;
        **receiver_account.try_borrow_mut_lamports()? += data.amount;
        escrow_data.lamports_withdrawn += data.amount;

        Ok(())
    }

    fn process_close(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;

        let mut escrow_data = StreamData::try_from_slice(&escrow_account.data.borrow())
            .expect("failed to serialize escrow data");

        if escrow_data.sender != *sender_account.key {
            return Err(ProgramError::IllegalOwner);
        }

        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let time: i64 = Clock::get()?.unix_timestamp;
        let mut lamports_streamed_to_receiver: u64 = 0;

        if time > escrow_data.start_time {
            lamports_streamed_to_receiver = escrow_data.amount_second
                * (std::cmp::min(time, escrow_data.end_time) - escrow_data.start_time) as u64
                - escrow_data.lamports_withdrawn;
        }

        **receiver_account.try_borrow_mut_lamports()? += lamports_streamed_to_receiver;
        escrow_data.lamports_withdrawn += lamports_streamed_to_receiver;
        **sender_account.try_borrow_mut_lamports()? += **escrow_account.lamports.borrow();

        **escrow_account.try_borrow_mut_lamports()? = 0;

        Ok(())
    }
}
