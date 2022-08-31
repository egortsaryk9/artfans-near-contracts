### Contract initialization (new)

---

```
near call artfans-ft.test.near new '{"owner": "artfans-admin.test.near", "metadata": { "spec": "ft-1.0.0", "name": "Test Token", "symbol": "TST", "decimals": 24} }' --accountId artfans-admin.test.near

```


### Minting

---

#### Add minter (account that can mint new tokens, owner has this role by default). Operation is restricted to the contract owner

```
near call artfans-ft.test.near add_minter '{"account_id": "artfans-marketplace.test.near" }' --accountId artfans-admin.test.near

```

#### Remove minter. Operation is restricted to the contract owner

```
near call artfans_ft.test.near remove_minter '{ "account_id": "artfans-marketplace.test.near" }' --accountId artfans-admin.test.near

```

#### Mint tokens. Operation is restricted to minters

```
near call artfans-ft.test.near mint '{"account_id": "alice.test.near", "amount": "500000000000000000000000000"}' --accountId artfans-admin.test.near --amount 0.00125
```

#### Burn tokens. Operation is restricted to the contract owner

```
near call artfans-ft.test.near burn '{"account_id": "alice.test.near", "amount": "200000000000000000000000000"}' --accountId artfans-admin.test.near --depositYocto 1

```


### Fees for activity
---

#### Add fee collector (account that can charge fees in this token for activity). Operation is restricted to the contract owner

```
near call artfans-ft.test.near add_fee_collector '{"account_id": "artfans-social-network.test.near"}' --accountId artfans-admin.test.near
```

#### Remove fee collector. Operation is restricted to the contract owner

```
near call artfans-ft.test.near remove_fee_collector '{"account_id": "artfans-social-network.test.near"}' --accountId artfans-admin.test.near
```