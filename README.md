# IMPORTANT

this is only a POC there are many factors that one should take into account when running this
purpose of the repo is to send transaction from wallet A -> wallet B and check the delay in the grpc output

## PRE-build instructions

i don't take the private key via cli for now (todo)
you must edit https://github.com/Astralane/Astralane-basic-benchmark/blob/985935cc06ed4f1dd8bd06274e717c439eccd329/solana_txn/src/main.rs#L23 **and then build/run**

## build instructions

```
cargo build --release
```

once build is over head over to the targets/release folder and run solana_txn

you can also skip the building and running process via

```
cargo run -p solana_txn
```
