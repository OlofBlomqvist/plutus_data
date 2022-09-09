#![feature(box_into_inner)]

extern crate plutus_data_derive;
use std::collections::HashMap;

use cardano_multiplatform_lib::plutus::PlutusData;
pub use plutus_data_derive::FromPlutusDataDerive;
pub use plutus_data_derive::ToPlutusDataDerive;


// todo : handle things marked with base_16 attrib.
// todo : handle things marked with force variant.
pub trait ToPlutusData {
    fn to_plutus_data(&self) -> Result<PlutusData,String>;
}

// todo : handle things marked with base_16 attrib
pub trait FromPlutusData<T> {
    fn from_plutus_data(x:PlutusData) -> Result<T,String>;
}


impl<T1,T2> FromPlutusData<(T1,T2)> for (T1,T2) where T1: FromPlutusData<T1>, T2: FromPlutusData<T2> {
    fn from_plutus_data(x:PlutusData) -> Result<(T1,T2),String> {
        match x.as_list()  {
            Some(p) if p.len() == 2 => {
                let key_a_plutus_item = p.get(0);
                let key_b_plutus_item = p.get(1);
                Ok((T1::from_plutus_data(key_a_plutus_item)?,T2::from_plutus_data(key_b_plutus_item)?))
            },
            Some(_) => Err(String::from("Invalid number of items in tuple.")),
            None => {
                match x.as_constr_plutus_data() {
                    Some(p)=> {
                        let p = p.data();
                        if p.len() != 2 {
                            return Err(format!("expected tuple (list) with two items.. found {:?} with {} items.",x.kind(),p.len()))
                        }
                        let key_a_plutus_item = p.get(0);
                        let key_b_plutus_item = p.get(1);
                        Ok((T1::from_plutus_data(key_a_plutus_item)?,T2::from_plutus_data(key_b_plutus_item)?))
                    },
                    None => Err(format!("invalid tuple data.. {:?}: {:?}",x.kind(),x))
                }
            }
        }
    }
}


impl<T1,T2> ToPlutusData for (T1,T2) where T1: ToPlutusData , T2: ToPlutusData {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        let k = self.0.to_plutus_data()?;
        let v = self.1.to_plutus_data()?;
        let mut pl = cardano_multiplatform_lib::plutus::PlutusList::new();
        pl.add(&k);
        pl.add(&v);
        let cs = cardano_multiplatform_lib::plutus::ConstrPlutusData::new(
            &cardano_multiplatform_lib::ledger::common::value::BigNum::zero(), &pl);
        Ok(cardano_multiplatform_lib::plutus::PlutusData::new_constr_plutus_data(&cs))
    }
}


impl<K : ToPlutusData + Clone,V : ToPlutusData + Clone> ToPlutusData for HashMap<K,V> {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        let mut map = cardano_multiplatform_lib::plutus::PlutusMap::new();
        for kvp in self.iter() {
            let encoded_k = kvp.0.clone().to_plutus_data();
            let encoded_v = kvp.1.clone().to_plutus_data();
            map.insert(&encoded_k?,&encoded_v?);
        }
        Ok(cardano_multiplatform_lib::plutus::PlutusData::new_map(&map))
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Option<T>> for Option<T> {
    fn from_plutus_data(x:PlutusData) -> Result<Option<T>,String> {
        let result = T::from_plutus_data(x);
        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(format!("Failed to unpack option value from plutus data! Error: {}",e))
        }
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Box<T>> for Box<T> {
    fn from_plutus_data(x:PlutusData) -> Result<Box<T>,String> {
        let result = T::from_plutus_data(x);
        match result {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(format!("Failed to unpack option value from plutus data! Error: {}",e))
        }
    }
}

impl<T : ToPlutusData> ToPlutusData for Vec<T> {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        let mut vec_items = cardano_multiplatform_lib::plutus::PlutusList::new();
        for yyy in self {
            let vx = yyy.to_plutus_data();
            vec_items.add(&vx?);
        }
        Ok(
            cardano_multiplatform_lib::plutus::PlutusData::new_list(&vec_items)
        )
    }
}

impl<T1 :std::hash::Hash + std::cmp::Eq + FromPlutusData<T1>,T2 : FromPlutusData<T2>> FromPlutusData<HashMap<T1,T2>> for HashMap<T1,T2> {
    fn from_plutus_data(p:PlutusData) -> Result<HashMap<T1,T2>,String> {
        
        match p.as_map() {
            None => Err(format!("Attempting to decode a hashmap but instead found: {:?}.",p.kind())),
            Some(m) => {
                let items = m.keys();
                let mut result = HashMap::new();
                for n in 0 .. items.len() {
                    let the_key = items.get(n);
                    let k = T1::from_plutus_data(the_key.clone());
                    let the_value = m.get(&the_key).map_or(Err(String::from("found null value in plutus data. not supported")),|x|Ok(x))?;
                    let v = T2::from_plutus_data(the_value);
                    result.insert(k?,v?);
                }
                Ok(result)
            }
        }
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Vec<T>> for Vec<T> {
    fn from_plutus_data(p:PlutusData) -> Result<Vec<T>,String> {
        match p.as_list() {
            Some(pl) => {{
                let mut result : Vec<T> = vec![];
                for xi in 0 .. pl.len() {
                    match T::from_plutus_data(pl.get(xi)) {
                        Ok(v) => { result.push(v) },
                        Err(e) => return Err(format!("when decoding a vector, we got this exception: {}",e))
                    } 
                }
                Ok(result)
            }}
            None => Err(String::from("Failed to decode vec from plutus data because it was not a plutus list."))
        }
    }
}

impl FromPlutusData<String> for String {
    fn from_plutus_data(x:PlutusData) -> Result<String,String> {
        match x.as_bytes() {
            Some(bytes) => match std::str::from_utf8(&bytes) {
                Ok(s) => Ok(s.to_owned()),
                Err(_) => Ok(hex::encode(bytes))
            }  
            None => Err(format!("hmm... expected string bytes, found something else: {:?}..",x.kind()))
        }
    }
}

impl ToPlutusData for String {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        let bytes = String::as_bytes(self).to_vec();
        Ok(cardano_multiplatform_lib::plutus::PlutusData::new_bytes(bytes))
    }
}

impl<T: ToPlutusData> ToPlutusData for &Option<T> {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        match self {
            Some(v) => v.to_plutus_data(),
            None => Err(String::from("Not possible to encode None to plutus data.")),
        }
    }
}


impl<T: ToPlutusData> ToPlutusData for Option<T> {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        match self {
            Some(v) => v.to_plutus_data(),
            None => Err(String::from("Not possible to encode None to plutus data.")),
        }
    }
}


impl<T: ToPlutusData + Clone + ?Sized> ToPlutusData for Box<T> {
    fn to_plutus_data(&self) -> Result<PlutusData,String> {
        let inner_item : T = Box::into_inner(self.to_owned());
        inner_item.to_plutus_data()
    }
}


ImplPlutusForNum!(@i8);
ImplPlutusForNum!(@i16);
ImplPlutusForNum!(@i32);
ImplPlutusForNum!(@i64);
ImplPlutusForNum!(@i128);
ImplPlutusForNum!(@u8);
ImplPlutusForNum!(@u16);
ImplPlutusForNum!(@u32);
ImplPlutusForNum!(@u64);
ImplPlutusForNum!(@u128);
ImplPlutusForNum!(@usize);

mod macros {
    #[macro_export]
    #[doc(hidden)]
    macro_rules! ImplPlutusForNum {
        (@$T:ident) => {
            impl ToPlutusData for $T {
                fn to_plutus_data(&self) -> Result<PlutusData,String> {
                    match cardano_multiplatform_lib::ledger::common::value::BigInt::from_str(&self.to_string()) {
                        Ok(n) => Ok(cardano_multiplatform_lib::plutus::PlutusData::new_integer(&n)),
                        Err(_) => Err(format!("failed to parse {} to BigInt.",self)),
                    }
                }
            }
            impl ToPlutusData for &$T {
                fn to_plutus_data(&self) -> Result<PlutusData,String> {
                    match cardano_multiplatform_lib::ledger::common::value::BigInt::from_str(&self.to_string()) {
                        Ok(n) => Ok(cardano_multiplatform_lib::plutus::PlutusData::new_integer(&n)),
                        Err(_) => Err(format!("failed to parse {} to BigInt.",self)),
                    }
                }
            }
            impl FromPlutusData<$T> for $T {
                fn from_plutus_data(p:PlutusData) -> Result<$T,String> {
                    match p.as_integer() {
                        Some(vd) => {
                            match vd.to_str().parse::<$T>() {
                                Ok(vc) => Ok(vc),
                                Err(e) => Err(format!("Failed to convert string to number. {}.",e)),
                            }
                        },
                        None => Err(format!("failed to parse plutus data to num! input kind: {:?} - inner plutus data: {:?}",p.kind(),p)),
                    }
                }
            }
        }
    }
}