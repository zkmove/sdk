## CLI for zkMove Virtual Machine

### generate proof for the example

Before start, make sure you have a customized version of the Move CLI installed:

```shell
cargo install --git https://github.com/zkmove/aptos-core move-cli --branch witnessing
```

Build and publish the example. Then generate the witness while executing the example. By default, the witness will be
generated in a directory called `witnesses`.

```shell
move build
move sandbox publish --skip-fetch-latest-git-deps --ignore-breaking-changes
move sandbox run --skip-fetch-latest-git-deps --witness storage/0x0000000000000000000000000000000000000000000000000000000000000001/modules/fibonacci.mv test_fibonacci --args 10u64
```

Finally, execute the “zkmove run” command, which will run the full sequence of setup, proving and verification. Upon
successful execution, it will also report the proof size, proving time, and verification time.

```shell
# Don't forget to replace the witness filename with your own.
zkmove run -p example -w witnesses/test_fibonacci-1733485309514.json
```


### build publish-circuit aptos txn

```shell
cargo run -- --param-path challenge_0078-kzg_bn254_16.srs -k 12 aptos --verifier-address a9f85ec000d6b7e78aa006f0fe0fcb3f8b82b71262283b84f2434441318064e1 --package_dir example/ build-publish-vk-aptos-txn --entry_module 0x1::fibonacci --function_name test_fibonacci --max_rows 2048
```

### build verify-proof aptos txn
