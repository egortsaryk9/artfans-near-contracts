### Contract initialization (new)

---

```
near call artfans_marketplace.test.near new '{"owner": "artfans_admin.test.near", "activity_ft": "artfans_ft.test.near", "activity_ft_beneficiary": "artfans_social_network.test.near", "artfans_nft": "artfans_nft.test.near", "artfans_nft_beneficiary": "bank.test.near" }' --accountId artfans_admin.test.near
```


### Market

---

#### Buy activity FT

```
near call artfans_marketplace.test.near buy_activity_ft '' --accountId alice.test.near --amount 1
```

#### Mint NFT

```
near call artfans_marketplace.test.near mint_artfans_nft '{"token_id": "token_number_one"}' --accountId alice.test.near --amount 3.5
```