use std::collections::HashMap;

use plutus_data::FromPlutusDataDerive;
use plutus_data::ToPlutusDataDerive;


#[derive(ToPlutusDataDerive,FromPlutusDataDerive,Clone,Debug)]
pub struct SomeOtherStruct {
    pub example: String
}

#[derive(ToPlutusDataDerive,FromPlutusDataDerive,Clone,Debug)]
pub struct ExampleStruct {
    pub example_c : HashMap<(String,i64),SomeOtherStruct>,
    pub test1 : SomeOtherStruct,
    pub test2 : Option<SomeOtherStruct>,
    pub test3 : Option<SomeOtherStruct>,
    // bools are normally encoded/decoded using constr 0 [] / constr 1 []
    pub example_a : bool,
    // but can also use integer representation
    #[repr_bool_as_num]
    pub example_b : bool,
    pub utf8_string : String,
    // for strings that should be encoded/decoded as hex bytes
    #[base_16]
    pub hex_string : String,
    pub opt_test1 : Option<String>,
     // normally option values will be represented as constr 0 [value] (Some), or constr 1 [] (None).
    // using ignore container, they will not be wrapped inside constr. this also means that
    // you cannot encode None values when using that attribute.
    #[ignore_option_container]
    pub opt_test2 : Option<String>,
    
}

#[derive(Debug,ToPlutusDataDerive,FromPlutusDataDerive,Clone)]
enum EnumThing {
    A(ExampleStruct),
    B(String)
}

fn main() {
    let hello = EnumThing::A(ExampleStruct {
        example_c: HashMap::new(),
        test1: SomeOtherStruct{ example: String::from("example") },
        test2: Some(SomeOtherStruct{ example: String::from("weee") }),
        test3: None,
        example_a: true,
        example_b: false,
        utf8_string: "hello".into(),
        hex_string: "aabb".into(),
        opt_test1: None,
        opt_test2: Some("stringly".into())
    });
    let encoded = hello.to_plutus_data(&vec![]).unwrap();
    println!("ENCODED: {:?}",plutus_data::to_hex(&encoded).unwrap());
    let decoded = EnumThing::from_plutus_data(encoded,&vec![]).unwrap();
    println!("DECODED {:?}",decoded);
}