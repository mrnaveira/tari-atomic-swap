use sha2::Digest;
use sha2::Sha256;
use tari_template_lib::{prelude::*, Hash};

pub type Preimage = [u8; 32];

#[template]
mod atomic_swap_template {
    use super::*;

    pub struct HashedTimelock {
        locked_funds: Vault,
        sender_token: NonFungibleAddress,
        receiver_token: NonFungibleAddress,
        hashlock: Hash,
        preimage: Option<Preimage>,
        // TODO: we are using epoch number for now, but we will need block/timestamp support eventually
        timelock: u64,
    }

    impl HashedTimelock {
        pub fn create(
            funds: Bucket,
            sender_token: NonFungibleAddress,
            receiver_token: NonFungibleAddress,
            hashlock: Hash,
            timelock: u64,
        ) -> HashedTimelockComponent {
            // funds cannot be empty
            assert!(
                funds.amount() > Amount::zero(),
                "The bucket with the funds cannot be empty"
            );
            let locked_funds = Vault::from_bucket(funds);

            // check that the timelock is valid
            assert!(
                timelock > Consensus::current_epoch(),
                "The timelock must be in the future"
            );

            // only the owner of the receiver account will be able to withdraw funds by revealing the preimage
            let withdraw_rule = AccessRule::Restricted(Require(receiver_token.clone()));

            // and only the owner of the sender account will be able to refund after the timelock
            let refund_rule = AccessRule::Restricted(Require(sender_token.clone()));

            // enforce the security rules on the proper methods
            let rules = AccessRules::new()
                .add_method_rule("withdraw", withdraw_rule)
                .add_method_rule("refund", refund_rule);

            Self {
                locked_funds,
                sender_token: sender_token.clone(),
                receiver_token: receiver_token.clone(),
                hashlock,
                timelock,
                preimage: None,
            }
            .create_with_options(rules, None)
        }

        // called by the receiver of the swap, once they know the hashlock preimage, to retrieve the funds
        pub fn withdraw(&mut self, preimage: Preimage) -> Bucket {
            self.check_hashlock(&preimage);

            // we explicitly store the preimage to make it easier for the other party to retrieve it
            self.preimage = Some(preimage);
            self.locked_funds.withdraw_all()
        }

        // called by the sender of the swap to get back the funds if the swap failed
        pub fn refund(&mut self) -> Bucket {
            self.check_timelock();

            self.locked_funds.withdraw_all()
        }

        pub fn get_sender_public_key(&self) -> RistrettoPublicKeyBytes {
            self.sender_token
                .to_public_key()
                .unwrap_or_else(|| panic!("sender_token is not a valid public key: {}", self.sender_token))
        }

        pub fn get_receiver_public_key(&self) -> RistrettoPublicKeyBytes {
            self.receiver_token
                .to_public_key()
                .unwrap_or_else(|| panic!("receiver_token is not a valid public key: {}", self.receiver_token))
        }

        fn check_hashlock(&self, preimage: &Preimage) {
            let mut hasher = Sha256::new();
            hasher.update(preimage);
            let hashlock: [u8; 32] = hasher.finalize().into();
            let hashlock: Hash = hashlock.into();

            assert!(self.hashlock == hashlock, "Invalid preimage");
        }

        fn check_timelock(&self) {
            assert!(Consensus::current_epoch() > self.timelock, "Timelock not yet passed");
        }
    }
}
