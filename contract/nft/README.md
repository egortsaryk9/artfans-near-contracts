### Contract initialization (new)

---

```
near call artfans_nft.test.near new '{"owner": "artfans-admin.test.near", "contract_metadata": { "spec": "nft-1.0.0", "name": "Artfans NFT collection", "symbol": "ABC" }, "default_token_metadata": { "title": "Very beautiful NFT!" } }' --accountId artfans-admin.test.near
```


### Minting

---

#### Add minter (account that can mint new tokens, owner has this role by default). Operation is restricted to the contract owner

```
near call artfans_nft.test.near add_minter '{"account_id": "artfans-marketplace.test.near"}' --accountId artfans-admin.test.near
```

#### Remove minter. Operation is restricted to the contract owner

```
near call artfans_nft.test.near remove_minter '{ "account_id": "artfans-marketplace.test.near" }' --accountId artfans-admin.test.near

```

#### Mint token. Operation is restricted to minters

```
near call artfans_nft.test.near nft_mint '{ "token_id": "token_number_one", "receiver_id": "alice.test.near" }' --accountId artfans-admin.test.near --amount 0.01
```

### Token metadata updating

---

#### Add token metadata admin (account that can set/update token metadata, owner has this role by default). Operation is restricted to the contract owner

```
near call artfans_nft.test.near add_token_metadata_admin '{ "account_id": "alice.test.near" }' --accountId artfans-admin.test.near
```

#### Remove token metadata admin. Operation is restricted to the contract owner

```
near call artfans_nft.test.near remove_token_metadata_admin '{ "account_id": "alice.test.near" }' --accountId artfans-admin.test.near
```

#### Set token metadata. Opeartion is restricted to token metadata admins

```
near call artfans_nft.test.near set_token_metadata '{ "token_id": "token_number_one", "token_metadata": { "title": "Awesome NFT !", "description": "Some description" } }' --accountId artfans-admin.test.near --amount 0.01
```

#### Set default token metadata. Opeartion is restricted to token metadata admins

```
near call artfans_nft.test.near set_default_token_metadata '{ "default_token_metadata": { "title": "New default title", "description": "New default desription" } }' --accountId artfans-admin.test.near --amount 0.01
```

### Get token
```
local_near view artfans_nft5.test.near nft_token '{ "token_id": "token_number_one" }'
```
