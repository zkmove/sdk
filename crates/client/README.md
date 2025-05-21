## CLI for zkMove Virtual Machine

### generate proof for the example

Before start, make sure you have a customized version of the Move CLI installed:

```shell
cargo install --git https://github.com/zkmove/aptos-core move-cli --branch witnessing
```

Build and publish the example. Then generate the witness while executing the example. By default, the witness will be
generated in a directory called `witnesses`.

```shell
# Run below commands in crates/client/example/
move build
move sandbox publish --skip-fetch-latest-git-deps --ignore-breaking-changes
move sandbox run --skip-fetch-latest-git-deps --witness storage/0x0000000000000000000000000000000000000000000000000000000000000001/modules/fibonacci.mv test_fibonacci --args 10u64
```

Finally, execute the “zkmove run” command, which will run the full sequence of setup, proving and verification. Upon
successful execution, it will also report the proof size, proving time, and verification time.

```shell
# Generate proof. Run under crates/client/, don't forget to replace the witness filename with your own.
cargo run --release -- --param-path params/kzg_bn254_12.srs prove -w example/witnesses/test_fibonacci-1747793629098.json
```
