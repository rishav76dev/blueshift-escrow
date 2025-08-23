use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
     Mint, TokenAccount, transfer_checked, TransferChecked, TokenInterface,
    },
};

use crate::state::Escrow;



// This struct defines the accounts required for the `make` instruction
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
  #[account(mut)]
  pub maker: Signer<'info>,

  #[account(// The escrow account that will hold details about this trade
    init,
    seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
    bump,
    payer = maker,
    space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
  )]
  pub escrow: Account<'info, Escrow>,

  // The two mints involved in the escrow trade
  #[account(
    mint::token_program = token_program
  )]
  pub mint_a: InterfaceAccount<'info, Mint>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint_b: InterfaceAccount<'info, Mint>,

  // The maker's token account for mint_a (from which tokens will be deposited)
  #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = maker,
    associated_token::token_program = token_program
  )]
  pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

  // Vault account owned by the escrow where tokens will be temporarily stored
  #[account(
    init,
    payer = maker,
    associated_token::mint = mint_a,
    associated_token::authority = escrow,
    associated_token::token_program = token_program
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}


impl<'info> Make<'info> {
  // This function populates the escrow account with initial data
  pub fn populate_escrow(&mut self, seed: u64, amount: u64, bump: u8) -> Result<()> {
    self.escrow.set_inner(Escrow{
      seed,
      maker: self.maker.key(),
      mint_a: self.mint_a.key(),
      mint_b: self.mint_b.key(),
      receive: amount,
      bump
    });
    Ok(())
  }
  // This function transfers tokens from the maker to the escrow vault
  pub fn deposit_tokens(&self, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;
      Ok(())


  }


}
