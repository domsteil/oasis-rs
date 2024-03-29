use map_vec::Map; // Provides a Map-like API but with smaller constant factors.
use oasis_std::{Address, Context};

// Each service definition contains a struct that derives `Service`.
// This struct represents the service's persistent state.
// Changes to the state will be persisted across service invocations.
#[derive(oasis_std::Service)]
pub struct Adappt {
    applicationId: String,
    description: String,
    accepted: bool,
    policy_holder: Address,
    agent: Address,
}

type Result<T> = std::result::Result<T, String>; // define our own result type, for convenience

impl Adappt {
    /// Constructs a new `Adappt`.
    pub fn new(ctx: &Context, applicationId: String, description: String) -> Result<Self> {
        Ok(Self {
            applicationId,
            description,
            accepted: false,
            policy_holder: ctx.sender(),
            agent: ctx.receiver(),
        })
    }

    /// Returns the id of the application. 
    pub fn applictionId(&self, _ctx: &Context) -> Result(&str> {
        Ok(&self.applicationId)
    }

    /// Returns the description of this application.
    pub fn description(&self, _ctx: &Context) -> Result<&str> {
        Ok(&self.description)
    }

    /// Returns the candidates being voted upon.
    pub fn candidates(&self, _ctx: &Context) -> Result<Vec<&str>> {
        Ok(self.candidates.iter().map(String::as_ref).collect())
    }

    // Accept the application for the candidate
    pub fn accept(&self, _ctx: &Context) -> Result<&bool> {
        if self.accepted = true {
            return Err("The application has already been approved".to_string());
        }

        self.accepted = true;
        Ok(())
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

    /// Cast a vote for a candidate.
    /// `candidate_num` is the index of the chosen candidate in `Ballot::candidates`.
    /// If you have already voted, this will change your vote to the new candidate.
    /// Voting for an invalid candidate or after the ballot has closed will return an `Err`.
    pub fn vote(&mut self, ctx: &Context, candidate_num: u32) -> Result<()> {
        if !self.accepting_votes {
            return Err("Voting is closed.".to_string());
        }
        if candidate_num as usize >= self.candidates.len() {
            return Err(format!("Invalid candidate `{}`.", candidate_num));
        }
        if let Some(prev_vote) = self.voters.insert(ctx.sender(), candidate_num) {
            self.tally[prev_vote as usize] -= 1;
        }
        self.tally[candidate_num as usize] =
            self.tally[candidate_num as usize].checked_add(1).unwrap();
        Ok(())
    }

    /// Closes this ballot so that it no longer collects votes.
    /// Only the ballot creator can close voting.
    pub fn close(&mut self, ctx: &Context) -> Result<()> {
        if self.admin != ctx.sender() {
            return Err("You cannot close the ballot.".to_string());
        }
        self.accepting_votes = false;
        Ok(())
    }

    /// Returns the index of the candidate with the most votes.
    /// This method can only be called after voting has closed.
    pub fn winner(&self, _ctx: &Context) -> Result<u32> {
        if self.accepting_votes {
            return Err("Voting is not closed.".to_string());
        }
        Ok(self
            .tally
            .iter()
            .enumerate()
            .max_by_key(|(_i, v)| *v)
            .unwrap()
            .0 as u32)
    }
}

fn main() {
    oasis_std::service!(Adappt);
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
