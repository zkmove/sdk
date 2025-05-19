# zkmove-contracts

aptos move contracts for zkmove sdk

### build

``` shell
aptos move compile --named-addresses zkmove=0x1234 --skip-fetch-latest-git-deps
```

### deploy

first you need to fund your account with `aptos account fund-with-faucet`, then create resource account and publish packages.

```shell
aptos move create-resource-account-and-publish-package --address-name zkmove --seed zkmove --skip-fetch-latest-git-deps
```