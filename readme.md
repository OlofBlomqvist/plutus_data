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
    pub opt_test : Option<String>,
    // normally option values will be represented as constr 0 [value] (Some), or constr 1 [] (None).
    // using ignore container, they will not be wrapped inside constr. this also means that
    // you cannot encode None values when using that attribute.
    #[attr(ignore_option_container)]
    pub opt_test2 : Option<String>
    
}

#[derive(Debug,ToPlutusDataDerive,FromPlutusDataDerive)]
enum EnumThing {
    A(Test),
    B(String)
}

fn main() {
    let hello = EnumThing::A(Test { 
        must_be_hex_string : String::from("AA"),
        bool: true,
        bool_as_num: true,
        option_unwrapped: Some(42),
        option_wrapped: Some(42)
    });
    let encoded = hello.to_plutus_data(&vec![]).unwrap();
    println!("ENCODED: {:?}",encoded);
    let decoded = EnumThing::from_plutus_data(encoded,&vec![]).unwrap();
    println!("DECODED {:?}",decoded);
}

```
```text

ENCODED: PlutusData { datum: ConstrPlutusData(ConstrPlutusData { alternative: BigNum(0), data: PlutusList { elems: [PlutusData { datum: ConstrPlutusData(ConstrPlutusData { alternative: BigNum(0), data: PlutusList { elems: [PlutusData { datum: Bytes([170]), original_bytes: None }, PlutusData { datum: ConstrPlutusData(ConstrPlutusData { alternative: BigNum(1), data: PlutusList { elems: [], definite_encoding: None } }), original_bytes: None }, PlutusData { datum: Integer(BigInt(1)), original_bytes: None }, PlutusData { datum: ConstrPlutusData(ConstrPlutusData { alternative: BigNum(0), data: PlutusList { elems: [PlutusData { datum: Integer(BigInt(42)), original_bytes: None }], definite_encoding: None } }), original_bytes: None }, PlutusData { datum: Integer(BigInt(42)), original_bytes: None }], definite_encoding: None } }), original_bytes: None }], definite_encoding: None } }), original_bytes: None }   

DECODED A(Test { must_be_hex_string: "aa", bool: true, bool_as_num: true, option_wrapped: Some(42), option_unwrapped: Some(42) })
```


