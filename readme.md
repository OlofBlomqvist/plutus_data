## Plutus Data

Small proc macro implementation for making it easier to work
with plutus data in rust.

```rust
#[derive(ToPlutusDataDerive,FromPlutusDataDerive)]
pub struct ExampleStruct {
    pub example_c : HashMap<(String,i64),SomeOtherStruct>
    pub state : MarloweDatumState,
    pub contract : Contract,
    // bools are normally encoded/decoded using constr 0 [] / constr 1 []
    pub example_a : bool,
    // but can also use integer representation
    #[attr(repr_bool_as_num)]
    pub example_b : bool,
    pub utf8_string : String,
    // for strings that should be encoded/decoded as hex bytes
    #[attr(base_16)]
    pub hex_string : String,
    
}
#[derive(ToPlutusDataDerive,FromPlutusDataDerive)]
pub enum Example {
    VariantOne(MarloweDatum),
    // if you always need this enum to be encoded/decoded using the
    // inner type of a specific variant.
    #[attr(force_variant)]
    VariantTwo(String)
}
```


```rust
// Encoding to plutus:
|x:MarloweDatum| {
    x.to_plutus_data(vec![])
}

// Decoding from plutus:
MarloweDatum::from_plutus_data(x,vec![])
```



