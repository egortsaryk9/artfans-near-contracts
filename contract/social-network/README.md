### Contract initialization (new)

---

```
near call artfans-social-network.test.near new '{"owner": "artfans-admin.test.near", "fee_ft": "artfans-ft.test.near", "settings": { "account_recent_likes_limit": 5, "add_message_extra_fee_percent": 20, "like_post_extra_fee_percent": 20, "like_message_extra_fee_percent": 20, "add_friend_extra_fee_percent": 20, "update_profile_extra_fee_percent": 20, "account_recent_like_extra_fee_percent": 20 } }' --accountId artfans-admin.test.near
```

### Post messages (comments)

---

#### Add message

```
near call artfans-social-network.test.near add_message_to_post '{"post_id": "post_number_one", "text": "This is a test message"}' --accountId alice.test.near
```

#### Get message by ID

```
near view artfans-social-network.test.near get_post_message '{"msg_id": { "post_id": "post_number_one", "msg_idx": "0"}}'
```

#### Get messages list for a post

```
near view artfans-social-network.test.near get_post_messages '{"post_id": "post_number_one", "from_index": "0", "limit": "100"}'
```

#### Add Nested message (comment reply)

```
near call artfans-social-network.test.near add_message_to_message '{"parent_msg_id": { "post_id": "post_number_one", "msg_idx": "0"}, "text": "This is a nested message"}' --accountId alice.test.near
```

### Likes

---

#### Like post

```
near call artfans-social-network.test.near like_post '{ "post_id": "post_number_one" }' --accountId bob.test.near
```

#### Unlike post

```
near call artfans-social-network.test.near unlike_post '{ "post_id": "post_number_two" }' --accountId bob.test.near
```

#### Get likes for post

```
near view artfans-social-network.test.near get_post_likes '{ "post_id": "post_number_one", "from_index": "0", "limit": "100" }'
```

#### Like message

```
near call artfans-social-network.test.near like_message '{ "msg_id": { "post_id": "post_number_one", "msg_idx": "0"} }' --accountId bob.test.near
```

#### Unlike message

```
near call artfans-social-network.test.near unlike_message '{ "msg_id": { "post_id": "post_number_one", "msg_idx": "0"} }' --accountId bob.test.near
```

#### Get likes for message

```
near view artfans-social-network.test.near get_message_likes '{ "msg_id": { "post_id": "post_number_one", "msg_idx": "1"}, "from_index": "0", "limit": "100" }'
```

#### Get account last likes

```
near view artfans-social-network.test.near get_account_last_likes '{"account_id": "alice.test.near", "from_index": "0", "limit": "100"}'
```

### Friends

---

#### Add friend

```
near call artfans-social-network.test.near add_friend '{"friend_id": "alice.test.near"}' --accountId bob.test.near
```

#### Get account friends

```
near view artfans-social-network.test.near get_account_friends '{"account_id": "alice.test.near", "from_index": "0", "limit": "100"}'
```

### Profiles

---

#### Update profile

```
near call artfans-social-network.test.near update_profile '{"profile": { "json_metadata": "{ \"name\": \"Alice Lee\", \"age\": 32 }", "image_url": "http:://some-resource" } }' --accountId alice.test.near
```

#### Get profile

```
near view artfans-social-network.test.near get_profile '{"account_id": "alice.test.near"}'
```

### Networks Settings

---

#### Update network settings. Operation is restricted to the contract owner

```
near call artfans-social-network.test.near update_admin_settings '{"settings": { "account_recent_likes_limit": 5, "add_message_extra_fee_percent": 20, "like_post_extra_fee_percent": 20, "like_message_extra_fee_percent": 20, "add_friend_extra_fee_percent": 20, "update_profile_extra_fee_percent": 20, "account_recent_like_extra_fee_percent": 20 } }' --accountId artfans-admin.test.near
```

#### Get network settings

```
near view artfans-social-network.test.near get_admin_settings ''
```
