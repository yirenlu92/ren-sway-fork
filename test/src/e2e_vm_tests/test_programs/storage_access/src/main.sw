contract;

storage {
    owner: ContractOwner =
        ContractOwner {
            data:
                OwnerInner {
                    value:  0x0000000000000000000000000000000000000000000000000000000000000000
                }
        },
    number: u64 = 0,
}

struct ContractOwner {
    data: OwnerInner,
}

struct OwnerInner {
    value: b256,
}


impure fn returns_owner() -> b256 {
    (storage.owner).data.value
}

impure fn set_owner(val: b256) {
    storage.owner.data.value = val;
}

abi TestAbi {
  fn test_deposit(unused: u64, unused: u64, unused: b256, val: u64) -> b256;
  fn set_owner(unused: u64, unused: u64, unused: b256, new_owner: b256);
}

impl TestAbi for Contract {
  impure fn test_deposit(unused: u64, unused: u64, unused: b256, val: u64) -> b256 {
    returns_owner()
  }
  impure fn set_owner(unused: u64, unused: u64, unused: b256, val: b256) {
      set_owner(val)
  }
}
