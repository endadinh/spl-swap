pub const POOL_SEED_1: &[u8] = &[101, 191, 209, 12, 36, 241, 255, 11];
pub const SIGNER_SEED_1: &[u8] = &[240, 191, 125, 12, 36, 152, 255, 16];


use anchor_lang::{prelude::*, solana_program::{instruction::Instruction, program::{invoke, invoke_signed}, system_instruction }};
use anchor_spl::token::{Mint,MintTo,Token,mint_to,Transfer,transfer};


#[error_code] 
pub enum ErrorCode { 
    #[msg("SwapPool: Unauthorized. ")]
    Unauthorized = 401
}
declare_id!("Gc45jvXbsUf1f7rM1ZqfxaM3FD3owJJ6opnWssbwsKFu");


#[program]
pub mod token_swap {
    
    use super::*;
    
    pub fn init_pool(_ctx: Context<CreatePool>) -> Result<()> {
        let owner = &_ctx.accounts.owner;
        let pool_account = &mut _ctx.accounts.pool_account;
        let (_, signer_nonce) = Pubkey::find_program_address(
            &[
              &SIGNER_SEED_1,
              pool_account.to_account_info().key.as_ref(),  
            ],
            _ctx.program_id,
          );
        pool_account.signer_nonce = signer_nonce;   
        pool_account.owner = *owner.key;
        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
      // Create the MintTo struct for our context
      let cpi_accounts = MintTo {
          mint: ctx.accounts.mint_account_a.to_account_info(),
          to: ctx.accounts.pool_token_account_a.to_account_info(),
          authority: ctx.accounts.authority.to_account_info(),
      };

      let cpi_accounts_2 = MintTo { 
        mint : ctx.accounts.mint_account_b.to_account_info(),
        to: ctx.accounts.pool_token_account_b.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
      };
      let cpi_program = ctx.accounts.token_program.clone();
      // Create the CpiContext we need for the request
      let cpi_ctx = CpiContext::new(cpi_program.to_account_info(), cpi_accounts);
      let cpi_ctx_2 = CpiContext::new(cpi_program.to_account_info(),cpi_accounts_2);
      // Execute anchor's helper function to mint tokens
      mint_to(cpi_ctx, amount)?;
      mint_to(cpi_ctx_2, amount)?;
      Ok(())
    }




    #[access_control(is_owner(&_ctx.accounts.admin.key, &_ctx.accounts.pool_account ))]
    pub fn setting_rate(_ctx: Context<SetRate>, _new_rate: u64) -> Result<()> {
        let pool_account =  &mut _ctx.accounts.pool_account;
        pool_account.rate = _new_rate;
        Ok(())
    }

    pub fn swap_token_to_sol(_ctx: Context<SwapTokenToSol>, amount: u64) -> Result<()> { 
        let pool_account = &_ctx.accounts.pool_account;
        let pool_signer = &_ctx.accounts.pool_signer;
        let payer = &_ctx.accounts.swapper;
        let swapper_ata_token = &_ctx.accounts.swapper_ata_token;
        let pool_ata_token = &_ctx.accounts.pool_ata_token;
        
        let cpi_program = _ctx.accounts.token_program.clone();
        let cpi_accounts = Transfer { 
          from : swapper_ata_token.to_account_info(),
          to: pool_ata_token.to_account_info(),
          authority: payer.to_account_info()
        };

        let ctx_cpi = CpiContext::new(
          cpi_program.to_account_info(),
          cpi_accounts);        
        transfer(ctx_cpi, amount)
        .expect("Pool_error: CPI fail");


        let receive_amount = amount * pool_account.rate / 100;

        let seeds: &[&[_]] = &[
          &SIGNER_SEED_1,
          pool_account.to_account_info().key.as_ref(),   
          &[pool_account.signer_nonce],
        ];
        // let seeds = &[];
        transfer_lamport(
          pool_signer, 
          payer, 
          receive_amount, 
          &[&seeds])
          // &[&seeds])
          

        .expect("Pool_error: CPI Fail");
        Ok(())
    } 


    pub fn swap_sol_to_token(_ctx: Context<SwapTokenToSol>, amount: u64) -> Result<()> { 
      let pool_account = &_ctx.accounts.pool_account;
      let pool_signer = &_ctx.accounts.pool_signer;
      let payer = &_ctx.accounts.swapper;
      let swapper_ata_token = &_ctx.accounts.swapper_ata_token;
      let pool_ata_token = &_ctx.accounts.pool_ata_token;
      
      let cpi_program = _ctx.accounts.token_program.clone();
      let seeds = &[];
      transfer_lamport(
        payer, 
        pool_signer, 
        amount, 
        seeds)
        // &[&seeds])
      .expect("Pool_error: Send Fail");

      let receive_amount = amount * pool_account.rate / 100;

      let cpi_accounts_resend = Transfer { 
        from : pool_ata_token.to_account_info(),
        to: swapper_ata_token.to_account_info(),
        authority: pool_signer.to_account_info(),
      };

      let seeds: &[&[_]] = &[
        &SIGNER_SEED_1,
        pool_account.to_account_info().key.as_ref(),   
        &[pool_account.signer_nonce],
      ];
      let seeded = &[seeds];

      let ctx_cpi = CpiContext::new_with_signer(
        cpi_program.to_account_info(),
        cpi_accounts_resend,
        seeded,
      );
      transfer(ctx_cpi, receive_amount)
      .expect("Pool_error: CPI fail");
      Ok(())
  } 




    pub fn swap_token(_ctx: Context<SwapToken>, amount: u64) -> Result<()> { 
        let pool_account = &_ctx.accounts.pool_account;
        let pool_signer = &_ctx.accounts.pool_signer;
        let payer = &_ctx.accounts.swapper;
        let swapper_ata_token_a = &_ctx.accounts.swapper_ata_token_a;
        let swapper_ata_token_b = &_ctx.accounts.swapper_ata_token_b;
        let pool_ata_token_a = &_ctx.accounts.pool_ata_token_a;
        let pool_ata_token_b = &_ctx.accounts.pool_ata_token_b;
        
        let cpi_program = _ctx.accounts.token_program.clone();
        let cpi_accounts = Transfer { 
          from : swapper_ata_token_a.to_account_info(),
          to: pool_ata_token_a.to_account_info(),
          authority: payer.to_account_info()
        };

        let ctx_cpi = CpiContext::new(
          cpi_program.to_account_info(),
          cpi_accounts);        
        transfer(ctx_cpi, amount)
        .expect("Pool_error: CPI fail");


        let receive_amount = amount * pool_account.rate / 100;

        let cpi_accounts_resend = Transfer { 
          from : pool_ata_token_b.to_account_info(),
          to: swapper_ata_token_b.to_account_info(),
          authority: pool_signer.to_account_info(),
        };

        let seeds: &[&[_]] = &[
          &SIGNER_SEED_1,
          pool_account.to_account_info().key.as_ref(),   
          &[pool_account.signer_nonce],
        ];
        let seeded = &[seeds];

        let ctx_cpi_2 = CpiContext::new_with_signer(
          cpi_program.to_account_info(),
          cpi_accounts_resend,
          seeded,
        );
        transfer(ctx_cpi_2, receive_amount)
        .expect("Pool_error: CPI fail");
        Ok(())
    } 
  
}

#[derive(Accounts)]
pub struct CreatePool<'info> {
  #[account(signer,mut)]
  /// CHECK:` doc comment explaining why no checks through types are necessary.
  pub owner: AccountInfo<'info>,
    #[account( init,
      payer = owner,
      space = 16 + 40,
      seeds = [  
        &POOL_SEED_1, 
      ],
      bump
    )]
    pub pool_account: Account<'info, Pool>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct SwapToken<'info> {
  ///CHECK : Swapper
    #[account(signer)]
    pub swapper: AccountInfo<'info>,
    pub pool_account: Account<'info, Pool>,
    /// CHECK : Signer hold pool's assets
    #[account(
        mut,
        seeds = [ 
            &SIGNER_SEED_1,
            pool_account.to_account_info().key.as_ref(), 
            ],
        bump = pool_account.signer_nonce
        )]
    pub pool_signer: AccountInfo<'info>,
    /// CHECK : Swapper's TokenA account
    #[account(mut)]
    pub swapper_ata_token_a: AccountInfo<'info>,
    /// CHECK : Swapper's TokenB account
    #[account(mut)]
    pub swapper_ata_token_b: AccountInfo<'info>,
    /// CHECK : Token A
    #[account(mut)]
    pub token_a: Account<'info,Mint>,
    /// CHECK: Token B
    #[account(mut)]
    pub token_b: Account<'info,Mint>,
    /// CHECK: Pool's Token A Account
    #[account(mut)]
    pub pool_ata_token_a: AccountInfo<'info>,
    /// CHECK: Pool's Token B Account
    #[account(mut)]
    pub pool_ata_token_b: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,

}




#[derive(Accounts)]
pub struct SwapTokenToSol<'info> {
  ///CHECK : Swapper
    #[account(signer)]
    pub swapper: AccountInfo<'info>,
    pub pool_account: Account<'info, Pool>,
    /// CHECK : Signer hold pool's assets
    #[account(
        mut,
        seeds = [ 
            &SIGNER_SEED_1,
            pool_account.to_account_info().key.as_ref(), 
            ],
        bump = pool_account.signer_nonce
        )]
    pub pool_signer: AccountInfo<'info>,
    /// CHECK : Swapper's TokenA account
    #[account(mut)]
    pub swapper_ata_token: AccountInfo<'info>,
    /// CHECK : Swapper's TokenB account
    #[account(mut)]
    pub token: Account<'info,Mint>,
    /// CHECK: Token B
    #[account(mut)]
    pub pool_ata_token: AccountInfo<'info>,
    /// CHECK: Pool's Token B Account
    pub token_program: Program<'info, Token>,
  
    pub system_program: Program<'info, System>,


}




#[derive(Accounts)] 
pub struct SetRate<'info> { 
  /// CHECK: vault owner, verified using #access_control
    #[account(signer)]
    pub admin : AccountInfo<'info>,
    #[account(mut)]
    pub pool_account: Account<'info, Pool>,
}



#[account]
#[derive(Default)]
pub struct Pool {
    pub owner: Pubkey,
    pub signer_nonce: u8,
    pub rate: u64,
}

pub fn is_owner(user: &Pubkey, pool: &Pool) -> Result <() > { 
    require_keys_eq!(*user, pool.owner, ErrorCode::Unauthorized);
    Ok(())
}



#[derive(Accounts)]
pub struct MintToken<'info> {
   /// CHECK: This is the token that we want to mint (Token A)
   #[account(mut)]
   pub mint_account_a: Account<'info, Mint>,
   /// CHECK: This is the token that we want to mint (Token B)
   #[account(mut)]
   pub mint_account_b: Account<'info, Mint>,
   
   pub token_program: Program<'info, Token>,
   /// CHECK: This is the token account that we want to mint tokens to
   #[account(mut)]
   pub pool_token_account_a: AccountInfo<'info>,
   /// CHECK: This is the token account that we want to mint tokens to
   #[account(mut)]
   pub pool_token_account_b: AccountInfo<'info>,
   /// CHECK: the authority of the mint account
   pub authority: Signer<'info>,  
}
      



#[derive(AnchorSerialize, AnchorDeserialize, Default)]
pub struct TransferTokenParams {
  pub instruction: u8,
  pub amount: u64,
}

pub fn transfer_token<'a>(
  owner: &AccountInfo<'a>,
  from_pubkey: &AccountInfo<'a>,
  to_pubkey: &AccountInfo<'a>,
  amount: u64,
  signer_seeds: &[&[&[u8]]],
  program_id: &Program<'a,Token>
) -> std::result::Result<(), ProgramError> {
  let data = TransferTokenParams {
    instruction: 3,
    amount,
  };
  let instruction = Instruction {
    program_id: program_id.key(),
    accounts: vec![
      AccountMeta::new(*from_pubkey.key, false),
      AccountMeta::new(*to_pubkey.key, false),
      AccountMeta::new_readonly(*owner.key, true),
    ],
    data: data.try_to_vec().unwrap(),
  };
  if signer_seeds.len() == 0 {
    invoke(&instruction, &[from_pubkey.clone(), to_pubkey.clone(), owner.clone()])
  }
  else {
    invoke_signed(&instruction, &[from_pubkey.clone(), to_pubkey.clone(), owner.clone()], &signer_seeds)
  }
}


pub fn transfer_lamport<'a>(
  owner: &AccountInfo<'a>,
  recipient: &AccountInfo<'a>,
  amount: u64,
  signer_seeds: &[&[&[u8]]],
) -> std::result::Result<(), ProgramError> {

  let instruction = system_instruction::transfer(
    &owner.key,
    &recipient.key,
    amount,
  );
  if signer_seeds.len() == 0 {
    invoke(&instruction, &[owner.clone(), recipient.clone()])
  }
  else {
    invoke_signed(&instruction, &[owner.clone(), recipient.clone()], &signer_seeds)
  }
}