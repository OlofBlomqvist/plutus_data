## Plutus Data

Small proc macro implementation for making it easier to work
with plutus data in rust.

```rust
#[derive(ToPlutusDataDerive,FromPlutusDataDerive)]
pub struct MarloweDatum {
    pub state : MarloweDatumState,
    pub contract : Contract
}
```


```rust
// Encoding to plutus:
|x:MarloweDatum| {
    x.to_plutus_data()
}
// Decoding from plutus:
MarloweDatum::from_plutus_data(x)
```
