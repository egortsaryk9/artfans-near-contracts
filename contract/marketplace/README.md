### Contract initialization (new)

---

```
near call artfans_marketplace.test.near new '{"owner": "artfans_admin.test.near", "activity_ft": "artfans_ft.test.near", "activity_ft_beneficiary": "artfans_social_network.test.near" }' --accountId artfans_admin.test.near
```


### Market

---

#### Buy activity FT

```
near call artfans_marketplace.test.near buy_activity_ft '' --accountId alice.test.near --amount 1
```
