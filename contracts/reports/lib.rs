#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reports {
    #[ink(storage)]
    pub struct Reports {}

    impl Reports {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn get(&self) {}
    }
}
