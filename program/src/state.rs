use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct CreateStreamInput {
    pub start_time: UnixTimestamp,  // Time at which the stream will start
    pub end_time: UnixTimestamp,  // Time at which the stream will end 
    pub receiver: Pubkey,  // Public key of the reciever
    pub lamports_withdrawn: u64,  // Keep track of the Lamports withdrawn by the reciever
    pub amount_second: u64,  // Number of Lamports transferred to the reciever every second
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct WithdrawInput {
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct StreamData {
    pub start_time: UnixTimestamp,
    pub end_time: UnixTimestamp, 
    pub receiver: Pubkey,
    pub lamports_withdrawn: u64,
    pub amount_second: u64,
    pub sender: Pubkey,
}

impl StreamData {
    pub fn new(data: CreateStreamInput, sender: Pubkey) -> Self {
        StreamData {
            start_time: data.start_time,
            end_time: data.end_time,
            receiver: data.receiver,
            lamports_withdrawn: data.lamports_withdrawn,
            amount_second: data.amount_second,
            sender: sender,
        }
    }
}




