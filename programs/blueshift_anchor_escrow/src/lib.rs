use anchor_lang::prelude::*;

mod state;
mod error;
mod instructions;
use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        //...
         require_gt!(receive, 0, EscrowError::InvalidAmount);
    require_gt!(amount, 0, EscrowError::InvalidAmount);

    // Save the Escrow Data
    ctx.accounts.populate_escrow(seed, receive, ctx.bumps.escrow)?;

    // Deposit Tokens
    ctx.accounts.deposit_tokens(amount)?;

    Ok(())
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        //...
        ctx.accounts.transfer_to_maker()?;

    // Withdraw and close the Vault
    ctx.accounts.withdraw_and_close_vault()?;

    Ok(())
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        //...
Ok(())
    }
}
