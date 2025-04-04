use anchor_lang::prelude::*;
use solana_program::hash::Hash;

declare_id!("GEFoAn6CNJiG9dq8xgm24fjzjip7n5GcH5AyqVC6QzdD");

// Helper function for generating ciphertexts
fn generate_ciphertext(clock: &Clock) -> [u8; 32] {
    let timestamp = clock.unix_timestamp;
    let slot = clock.slot;
    
    let mut value = [0u8; 32];
    for i in 0..32 {
        let mixed = (
            (slot.wrapping_mul(1337 + i as u64)) ^
            (timestamp as u64).wrapping_mul(7919 + i as u64)
        ) as u8;
        value[i] = mixed;
    }
    value
}

#[program]
pub mod blockchain {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        // Get slot and recent blockhash for entropy
        let clock = Clock::get()?;
        let value = generate_ciphertext(&clock);
        
        ctx.accounts.deposit_info.owner = ctx.accounts.user.key();
        ctx.accounts.deposit_info.value = value;
        
        msg!("User {} deposited {} lamports", ctx.accounts.user.key(), amount);
        msg!("Deposit info: {:?}", ctx.accounts.deposit_info.value);
        msg!("Deposit info (hex): {:x?}", ctx.accounts.deposit_info.value);
        Ok(())
    }

    pub fn fhe8_add(ctx: Context<FHEOperation>, lhs: [u8; 32], rhs: [u8; 32]) -> Result<()> {
        msg!("FHE Add - LHS: {:?}", lhs);
        msg!("FHE Add - RHS: {:?}", rhs);
        
        let result_value = Hash::new_unique().to_bytes();
        
        ctx.accounts.result_info.owner = ctx.accounts.user.key();
        ctx.accounts.result_info.value = result_value;
        
        msg!("FHE addition result: {:?}", result_value);
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: [u8; 32], recipient: Pubkey) -> Result<()> {
        // Emit both sender's and recipient's ciphertext values
        msg!("Sender's deposit value: {:?}", ctx.accounts.sender_deposit.value);
        msg!("Recipient's deposit value: {:?}", ctx.accounts.recipient_deposit.value);
        msg!("Transferring {:?} from {:?} to {:?}", amount, ctx.accounts.user.key(), ctx.accounts.recipient.key());
        Ok(())
    }

    pub fn view_balance(ctx: Context<ViewBalance>) -> Result<[u8; 32]> {
        // Simply return the stored value bytes
        Ok(ctx.accounts.deposit_info.value) // check for signature
    }

    pub fn emit_bytes(ctx: Context<EmitBytes>, value: [u8; 32]) -> Result<()> {
        msg!("Emitting bytes: {:?}", value);
        Ok(())
    }

    pub fn encrypt(ctx: Context<FHEOperation>, value: [u8; 32]) -> Result<()> {
        msg!("Encrypting value: {:?}", value);
        
        ctx.accounts.result_info.owner = ctx.accounts.user.key();
        ctx.accounts.result_info.value = value;
        
        msg!("Encrypted result stored");
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, recipient: Pubkey) -> Result<()> {
        // Check owner
        require!(
            ctx.accounts.owner.key() == ctx.accounts.program_data.upgrade_authority_address.unwrap(),
            ProgramError::IncorrectProgramId
        );

        let recipient_starting_lamports = ctx.accounts.recipient.lamports();
        **ctx.accounts.recipient.lamports.borrow_mut() = recipient_starting_lamports
            .checked_add(amount)
            .unwrap();

        let vault_lamports = ctx.accounts.vault.lamports();
        **ctx.accounts.vault.lamports.borrow_mut() = vault_lamports
            .checked_sub(amount)
            .unwrap();

        msg!("Owner {} withdrew {} lamports to {}", 
            ctx.accounts.owner.key(),
            amount,
            recipient
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[account]
pub struct DepositInfo {
    owner: Pubkey,    // 32 bytes
    value: [u8; 32],  
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32, 
        seeds = [user.key().as_ref()],
        bump
    )]
    pub deposit_info: Account<'info, DepositInfo>,

    /// CHECK: This is the PDA that will hold SOL
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    pub vault: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(value: [u8; 32])]
pub struct Transfer<'info> {
    #[account(
        mut,
        seeds = [user.key().as_ref()],
        bump
    )]
    pub sender_deposit: Account<'info, DepositInfo>,

    #[account(
        mut,
        seeds = [recipient.key().as_ref()],
        bump
    )]
    pub recipient_deposit: Account<'info, DepositInfo>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: This is just for logging the recipient's address
    pub recipient: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmitBytes {}

#[derive(Accounts)]
pub struct ViewBalance<'info> {
    #[account(
        seeds = [user.key().as_ref()],
        bump,
    )]
    pub deposit_info: Account<'info, DepositInfo>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct FHEOperation<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32,
        seeds = [b"fhe", user.key().as_ref()],
        bump
    )]
    pub result_info: Account<'info, DepositInfo>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: SystemAccount<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    /// CHECK: Can be any account, owner decides
    pub recipient: UncheckedAccount<'info>,
    
    pub program_data: Account<'info, ProgramData>,
}