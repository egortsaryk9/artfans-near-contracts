### Deployment steps

##### 1st step

Deploy activity FT contract:

```
local_near deploy --wasmFile artfans_activity_ft.wasm --accountId artfans_ft.test.near
```

---

##### 2nd step

Initialize activity FT contract:

```
local_near call artfans_ft.test.near new '{"owner": "artfans_admin.test.near", "metadata": { "spec": "ft-1.0.0", "name": "Activity Token", "symbol": "TST", "decimals": 24} }' --accountId artfans_admin.test.near
```

---

##### 3rd step

Deploy social network contract:

```
local_near deploy --wasmFile artfans_social_network.wasm --accountId artfans_social_network.test.near
```

---

##### 4th step

Initialize social network contract:

```
local_near call artfans_social_network.test.near new '{"owner": "artfans_social_network.test.near", "fee_ft": "artfans_ft.test.near", "settings": { "account_recent_likes_limit": 5, "add_message_extra_fee_percent": 20, "like_post_extra_fee_percent": 20, "like_message_extra_fee_percent": 20, "add_friend_extra_fee_percent": 20, "update_profile_extra_fee_percent": 20, "account_recent_like_extra_fee_percent": 20 } }' --accountId artfans_admin.test.near
```

---

##### 5th step

Register social network contract as fee collector of activity FT:

```
local_near call artfans_ft.test.near add_fee_collector '{"account_id": "artfans_social_network.test.near"}' --accountId artfans_admin.test.near
```

---

##### 6th step

Deploy NFT contract:

```
local_near deploy --wasmFile artfans_nft.wasm --accountId artfans_nft.test.near
```

---

##### 7th step

Initialize NFT contract:

```
local_near call artfans_nft.test.near new '{"owner": "artfans_admin.test.near", "contract_metadata": { "spec": "nft-1.0.0", "name": "Artfans NFT collection", "symbol": "AAA" }, "default_token_metadata": { "title": "Default token title" } }' --accountId artfans_admin.test.near
```

---

##### 8th step

Deploy marketplace contract:

```
local_near deploy --wasmFile artfans_marketplace.wasm --accountId artfans_marketplace.test.near
```

---

##### 9th step

Initialize marketplace contract:

```
local_near call artfans_marketplace.test.near new '{"owner": "artfans_admin.test.near", "activity_ft": "artfans_ft.test.near", "activity_ft_beneficiary": "artfans_social_network.test.near", "artfans_nft": "artfans_nft.test.near", "artfans_nft_beneficiary": "bank.test.near" }' --accountId artfans_admin.test.near
```

* *activity_ft_beneficiary* - account, that will receive NEAR tokens when somebody buys 'activity_ft'
* *artfans_nft_beneficiary* - account, that will receive NEAR tokens, when somebody mints 'artfans_nft'

---

##### 10th step

Register the marketplace contract as activity FT minter:

```
local_near call artfans_ft.test.near add_minter '{"account_id": "artfans_marketplace.test.near" }' --accountId artfans_admin.test.near
```

---

##### 11th step

Register the marketplace contract as NFT minter:

```
local_near call artfans_nft.test.near add_minter '{"account_id": "artfans_marketplace.test.near"}' --accountId artfans_admin.test.near
```

---
