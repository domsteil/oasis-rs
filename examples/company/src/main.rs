use map_vec::Map; // Provides a Map-like API but with smaller constant factors.
use oasis_std::{Address, Context};

// Each service definition contains a struct that derives `Service`.
// This struct represents the service's persistent state.
// Changes to the state will be persisted across service invocations.
#[derive(oasis_std::Service)]
pub struct Company {
    companyId: String,
    companyName: String,
    companyType: String,
    industry: String,
    phone: String,
    email: String,
    description: String,
    controller: Address,
    processor: Address,
}

type Result<T> = std::result::Result<T, String>; // define our own result type, for convenience

impl Company {
    /// Constructs a new `Company`.
    pub fn new(ctx: &Context, companyId: String, companyName: String, companyType: String, industry: String, phone: String, email: String, description: String) -> Result<Self> {
        Ok(Self {
            companyId,
            companyName,
            companyType,
            industry,
            phone,
            email,
            controller: ctx.sender(),
            processor: ctx.receiver(),
        })
    }

    /// Returns the id of the account. 
    pub fn accountId(&self, _ctx: &Context) -> Result(&str> {
        Ok(&self.accountId)
    }

    /// Returns the description of this application.
    pub fn description(&self, _ctx: &Context) -> Result<&str> {
        Ok(&self.description)
    }



    // An Accept event struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, Event)]
pub struct Accept {
    #[indexed]
    pub admin: Address,
    #[indexed]
    pub agent: Address,
    #[indexed]
    pub applicationId: String,
}


fn main() {
    oasis_std::service!(Company);
}

#[cfg(test)]
mod tests {
    // This is required even in Rust 2018. If omitted, rustc will not link in the testing
    // library and will produce a giant error message.
    extern crate oasis_test;

    use super::*;

    /// Creates a new account and a `Context` with the new account as the sender.
    fn create_account() -> (Address, Context) {
        let addr = oasis_test::create_account(0 /* initial balance */);
        let ctx = Context::default().with_sender(addr).with_gas(100_000);
        (addr, ctx)
    }

    #[test]
    fn functionality() {
        let (_admin, admin_ctx) = create_account();
        let (_voter, voter_ctx) = create_account();

        let description = "What's for dinner?";
        let candidates = vec!["beef".to_string(), "yogurt".to_string()];
        let mut ballot =
            Ballot::new(&admin_ctx, description.to_string(), candidates.clone()).unwrap();

        assert_eq!(ballot.description(&admin_ctx).unwrap(), description);
        assert_eq!(ballot.candidates(&admin_ctx).unwrap(), candidates);

        // Can't get winner before voting has closed.
        assert!(ballot.winner(&voter_ctx).is_err());

        ballot.vote(&voter_ctx, 0).unwrap();
        ballot.vote(&voter_ctx, 1).unwrap();
        ballot.vote(&admin_ctx, 1).unwrap();

        // Non-admin can't close ballot.
        ballot.close(&voter_ctx).unwrap_err();
        ballot.close(&admin_ctx).unwrap();

        // Votes can't be cast after ballot has closed.
        ballot.vote(&admin_ctx, 0).unwrap_err();

        assert_eq!(ballot.winner(&voter_ctx).unwrap(), 1);
    }
}
