use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
     Mint, TokenAccount, close_account, CloseAccount, transfer_checked, TransferChecked,TokenInterface,
    },
};

// Import EscrowError from its module (update the path if needed)
use crate::error::EscrowError;
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    /// The taker is the user who wants to take the escrow trade.
    /// Needs to be a signer because they'll initiate token transfers.
  #[account(mut)]
  pub taker: Signer<'info>,

  /// The maker is the person who created the escrow.
    /// We mark it as `mut` so we can receive SOL from closing accounts.
  #[account(mut)]
  pub maker: SystemAccount<'info>,

  /// The Escrow account that holds the trade info.
  /// It has several checks:
  /// - `close = maker` → when escrow is done, close it and send lamports to maker.
  /// - `seeds` and `bump` → PDA for deterministic signing.
    /// - `has_one` checks → ensures the escrow account actually references the correct maker and mints.
 #[account(
  mut,
  close = maker,
  seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
  bump = escrow.bump,
  has_one = maker @ EscrowError::InvalidMaker,
  has_one = mint_a @ EscrowError::InvalidMintA,
  has_one = mint_b @ EscrowError::InvalidMintB,
 )]
 pub escrow: Box<Account<'info, Escrow>>,

 pub mint_a: Box<InterfaceAccount<'info, Mint>>,
 pub mint_b: Box<InterfaceAccount<'info, Mint>>,

 /// Vault account that holds Token A from the maker
/// This is owned by the escrow PDA.
 #[account(
  mut,
  associated_token::mint = mint_a,
  associated_token::authority = escrow,
  associated_token::token_program = token_program
 )]
 pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

 /// Taker's ATA (Associated Token Account) for receiving Token A
    /// If it doesn't exist, create it automatically (`init_if_needed`)
 #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = mint_a,
      associated_token::authority = taker,
      associated_token::token_program = token_program
  )]
  pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

  /// Taker's ATA for sending Token B to the maker
  #[account(
      mut,
      associated_token::mint = mint_b,
      associated_token::authority = taker,
      associated_token::token_program = token_program
  )]
  pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

  /// Maker's ATA for receiving Token B from the taker
 #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = mint_b,
      associated_token::authority = maker,
      associated_token::token_program = token_program
  )]
  pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,

}


impl<'info> Take<'info> {
    /// Transfers Token B from the taker to the maker
    pub fn transfer_to_maker(&mut self) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata_b.to_account_info(),
                    to: self.maker_ata_b.to_account_info(),
                    mint: self.mint_b.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            self.escrow.receive, // Amount of Token B to send
            self.mint_b.decimals,       // Decimal precision for Token B
        )?;

        Ok(())
    }

    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {
        // Create the signer seeds for the Vault
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Transfer Token A (Vault -> Taker)
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.taker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;

        // Close the Vaultand send SOL to the maker
        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                authority: self.escrow.to_account_info(),
                destination: self.maker.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())
    }
}


pub fn handler(ctx: Context<Take>) -> Result<()> {
    ctx.accounts.transfer_to_maker()?;

    ctx.accounts.withdraw_and_close_vault()?;

    Ok(())
}
