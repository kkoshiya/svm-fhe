use anchor_lang::prelude::*;
use blockchain::program::Blockchain;  // Import main program

declare_id!("7xpSX3HvqvTN7zVPZEm4WkeLpq5Trj9USRuH6vsYiJaz");  // Your program ID

#[program]
pub mod operations {
    use super::*;

    pub fn test_add(ctx: Context<TestOperation>, lhs: [u8; 32], rhs: [u8; 32]) -> Result<()> {
        // Call main program's fhe8_add
        let cpi_program = ctx.accounts.blockchain_program.to_account_info();
        let cpi_accounts = blockchain::cpi::accounts::FHEOperation {
            user: ctx.accounts.user.to_account_info(),
            result_info: ctx.accounts.result_info.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        blockchain::cpi::fhe8_add(cpi_ctx, lhs, rhs)?;

        msg!("Test add completed!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TestOperation<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// CHECK: This is the blockchain program
    pub blockchain_program: Program<'info, Blockchain>,
    
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32,
        seeds = [b"result", user.key().as_ref()],
        bump
    )]
    pub result_info: Account<'info, blockchain::DepositInfo>,  // Using DepositInfo from blockchain
    
    pub system_program: Program<'info, System>,
}