use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{
     Mint, TokenAccount, close_account, transfer_checked, CloseAccount, TransferChecked, TokenInterface,
    }
};

use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
  #[account(mut)]
  pub maker: Signer<'info>,

  #[account(mut,
    close = maker,
    has_one = maker,
    has_one = mint_a,
    has_one = mint_b,
    seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
    bump = escrow.bump,
  )]
   pub escrow: Box<Account<'info, Escrow>>,

  #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = maker,
    associated_token::token_program = token_program
  )]
  pub maker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

  #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = escrow,
    associated_token::token_program = token_program
  )]
  pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

  pub mint_a: Box<InterfaceAccount<'info, Mint>>,
  pub mint_b: Box<InterfaceAccount<'info, Mint>>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}

impl <'info> Refund<'info> {
  // vault -> maker_ata_a
pub fn process_refund(&mut self) -> Result<()> {
  let signer_seeds: [&[&[u8]]; 1] = [&[
    b"escrow",
    self.maker.to_account_info().key.as_ref(),
    &self.escrow.seed.to_le_bytes()[..],
    &[self.escrow.bump]
  ]];

  transfer_checked(
    CpiContext::new_with_signer(
      self.token_program.to_account_info(),
      TransferChecked{
        from: self.vault.to_account_info(),
        to: self.maker_ata_a.to_account_info(),
        mint: self.mint_a.to_account_info(),
        authority: self.escrow.to_account_info(),
      },
      &signer_seeds,
    ),
    self.vault.amount,
    self.mint_a.decimals,
  )?;

  close_account(CpiContext::new_with_signer(
    self.token_program.to_account_info(),
  CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.maker.to_account_info(),
                authority: self.escrow.to_account_info(),
            },
            &signer_seeds,
          ))?;

Ok(())
}
}
