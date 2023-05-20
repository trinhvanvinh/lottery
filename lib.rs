#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod lottery_test {
    use ink::env::hash::Keccak256;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct LotteryTest {
        /// Stores a single `bool` value on the storage.
        value: bool,
        owner: AccountId,
        running: bool,
        players: Vec<AccountId>,
        entries: Mapping<AccountId, Balance>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        LotteryNotRunning,
        CallerNotOwner,
        NoValueSent,
        ErrTransfer,
        PlayerAlreadyInLottery,
        NoEntries,
    }
    #[ink(event)]
    pub struct Entered {
        player: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Won {
        winner: AccountId,
        amount: Balance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl LotteryTest {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                value: init_value,
                owner: Self::env().caller(),
                running: false,
                players: Vec::new(),
                entries: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }
        #[ink(message)]
        pub fn pot(&self) -> Balance {
            self.env().balance()
        }
        #[ink(message)]
        pub fn is_running(&self) -> bool {
            self.running
        }
        #[ink(message)]
        pub fn get_players(&self) -> Vec<AccountId> {
            self.players.clone()
        }
        #[ink(message)]
        pub fn get_balances(&self, caller: AccountId) -> Option<Balance> {
            self.entries.get(caller)
        }

        fn seed(&self) -> u64 {
            let hash = self.env().hash_encoded::<Keccak256, _>(&self.players);
            let num = u64::from_be_bytes(hash[0..8].try_into().unwrap());
            let timestamp = self.env().block_timestamp();
            let block_number = self.env().block_number() as u64;
            num ^ timestamp ^ block_number
        }
        fn random(&self) -> u64 {
            let mut x = self.seed();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
        #[ink(message, payable)]
        pub fn enter(&mut self) -> Result<()> {
            if !self.running {
                return Err(Error::LotteryNotRunning);
            }
            let caller = self.env().caller();
            let balance = self.entries.get(caller);
            if balance.is_some() {
                return Err(Error::PlayerAlreadyInLottery);
            }
            let value = self.env().transferred_value();
            if value < 1 {
                return Err(Error::NoValueSent);
            }
            self.players.push(caller);
            self.entries.insert(caller, &value);
            self.env().emit_event(Entered {
                player: caller,
                value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn pick_winner(&mut self) -> Result<()> {
            if self.players.len() == 0 {
                return Err(Error::NoEntries);
            }
            let winner_index = self.random() % self.players.len() as u64;
            let winner = self.players[winner_index as usize];
            let amount = self.env().balance();

            if self.env().transfer(winner, amount).is_err() {
                return Err(Error::ErrTransfer);
            }
            for player in self.players.iter() {
                self.entries.remove(player);
            }
            self.players = Vec::new();
            self.env().emit_event(Won { winner, amount });
            Ok(())
        }
        #[ink(message)]
        pub fn start_lottery(&mut self) -> Result<()> {
            if self.env().caller() != self.owner{
                return Err(Error::CallerNotOwner)
            }
            self.running = true;
            Ok(())
        }

        #[ink(message)]
        pub fn stop_lottery(&mut self) -> Result<()> {
            if self.env().caller() != self.owner{
                return Err(Error::CallerNotOwner)
            }
            self.running = false;
            Ok(())
        }
    }
}
