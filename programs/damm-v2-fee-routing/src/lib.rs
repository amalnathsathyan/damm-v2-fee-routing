use anchor_lang::prelude::*;

declare_id!("5tihwv2Rv9oNUoiefeLim8sdTCDfuQfeHeXpdNt8WXCv");

#[program]
pub mod damm_v2_fee_routing {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
