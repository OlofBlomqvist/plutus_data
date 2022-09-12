## Plutus Data

Small proc macro implementation for making it easier to work
with plutus data in rust.

```rust
use plutus_data::ToPlutusDataDerive;
use plutus_data::FromPlutusDataDerive;

#[derive(Debug,ToPlutusDataDerive,FromPlutusDataDerive)]
struct Test {
    #[base_16]
    pub must_be_hex_string : String,
    pub bool : bool,
    #[repr_bool_as_num]
    pub bool_as_num : bool,
    pub option_wrapped : Option<i32>,
    #[ignore_container]
    pub option_unwrapped : Option<i32>
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


