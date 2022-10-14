pub const POOL_SEED_1: &[u8] = &[101, 191, 209, 12, 36, 241, 255, 11];
pub const SIGNER_SEED_1: &[u8] = &[101, 191, 209, 12, 36, 241, 255, 11];


use anchor_lang::{prelude::*, solana_program::{instruction::Instruction, program::{invoke, invoke_signed}}};

#[error_code] 
pub enum ErrorCode { 
    #[msg("SwapPool: Unauthorized. ")]
    Unauthorized = 401
}
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


#[program]
pub mod token_swap {
    
    use super::*;
    
    pub fn init_pool(_ctx: Context<CreatePool>) -> Result<()> {
        let owner = &_ctx.accounts.owner;
        let pool_account = &mut _ctx.accounts.pool_account;
        let (_, signer_nonce) = Pubkey::find_program_address(
            &[
              POOL_SEED_1,
              &pool_account.key().as_ref(),
            ],
            _ctx.program_id,
          );
        pool_account.signer_nonce = signer_nonce;   
        pool_account.owner = *owner.key;
        Ok(())
    }

  #[access_control(is_owner(&_ctx.accounts.admin.key, &_ctx.accounts.pool_account ))]
    pub fn setting_rate(_ctx: Context<SetRate>, _new_rate: u64) -> Result<()> {
        let pool_account =  &mut _ctx.accounts.pool_account;
        pool_account.rate = _new_rate;
        Ok(())
    }


    pub fn swap_token(_ctx: Context<SwapToken>, amount: u64) -> Result<()> { 
        let pool_account = &_ctx.accounts.pool_account;
        let pool_signer = &_ctx.accounts.pool_signer;
        // let pool_tok
        let sender = &_ctx.accounts.sender;
        // let recipient = &_ctx.accounts.recipient;
        let token_program_a = &_ctx.accounts.token_program_a;
        let token_program_b = &_ctx.accounts.token_program_b;
          let seeds: &[&[_]] = &[
            &SIGNER_SEED_1,
            pool_account.to_account_info().key.as_ref(),
            &[pool_account.signer_nonce],
          ];
        transfer_token(
            &pool_signer,
            &sender,
            &pool_signer,
            amount,
            &[&seeds],
            &token_program_a,
          )
          .expect("Pool_swap: CPI failed.");

          let receive_amount = amount * pool_account.rate / 100;
          transfer_token(
            &pool_signer,
            &pool_signer,
            &sender,
            receive_amount,
            &[&seeds],
            &token_program_b,
          )
          .expect("Pool_swap: CPI failed.");
        Ok(())
    } 
  
}
    // pub fn initialize(_ctx: Context<SwapToken>) -> Result<()> {

    //     Ok(())
    // }
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
            &pool_account.key().as_ref(),
          ],
          bump
        )]
        pub pool_account: Account<'info, Pool>,
        pub system_program: Program<'info, System>,
    }


#[derive(Accounts)]
pub struct SwapToken<'info> {

    /// CHECK: Transaction fee payer
    #[account(signer)]
    pub payer: AccountInfo<'info>,
    /// CHECK: Owner of source account
    #[account(mut)]
    // CHECK : pool account to hold access and sign
    pub pool_account: Account<'info, Pool>,
    #[account(
        seeds = [ 
            &SIGNER_SEED_1,
            pool_account. to_account_info().key.as_ref(), 
            ],
            bump = pool_account.signer_nonce
        )]

    /// CHECK : generate to sign tx
    pub pool_signer: AccountInfo<'info>,

    #[account(mut)]
    pub swapper: AccountInfo<'info>,
    #[account(mut)]
    pub sender: AccountInfo<'info>,
    /// CHECK: Destination token account
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(mut)]
    pub token_program_a: AccountInfo<'info>,
    #[account(mut)]
    pub token_program_b: AccountInfo<'info>,
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
    token_program: &AccountInfo<'a>
) -> std::result::Result<(), ProgramError > { 
    let data = TransferTokenParams {
        instruction: 3,
        amount,
      };
      let instruction = Instruction {
        program_id: *token_program.key,
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