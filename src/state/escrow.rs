use pinocchio::{AccountView};
use shank::ShankAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, ShankAccount)]
pub struct Escrow {
    pub mint_b: [u8; 32],
    pub amount_b: [u8; 8],
    pub seed: [u8; 1],
    pub bump: [u8; 1],
}
impl Escrow {
    pub const LEN: usize = 42;

    pub fn from_account_info_mut(account_info: &AccountView) -> Result<&mut Self, pinocchio::error::ProgramError> {
        let mut data = account_info.try_borrow_mut()?;

        if data.len() != Escrow::LEN {
            return Err(pinocchio::error::ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self)})
    }

    pub fn set_inner(&mut self, mint_b: [u8; 32], amount_b: [u8; 8], seed: [u8; 1], bump: [u8;1]) {
        self.mint_b = mint_b;
        self.amount_b = amount_b;
        self.seed = seed;
        self.bump = bump;
    }

}