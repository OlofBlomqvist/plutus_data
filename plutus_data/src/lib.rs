#![feature(box_into_inner)]

//extern crate plutus_data_derive;

use std::collections::HashMap;

use cardano_multiplatform_lib::ledger::common::value::BigInt;
use cardano_multiplatform_lib::ledger::common::value::BigNum;
use cardano_multiplatform_lib::plutus::ConstrPlutusData;
use cardano_multiplatform_lib::plutus::PlutusData;
use cardano_multiplatform_lib::plutus::PlutusList;
pub use plutus_data_derive::FromPlutusDataDerive;
pub use plutus_data_derive::ToPlutusDataDerive;


pub trait ToPlutusData {
    fn to_plutus_data(&self,attributes:&Vec<String>) -> Result<PlutusData,String>;
}

pub trait FromPlutusData<T> {
    fn from_plutus_data(x:PlutusData,attributes:&Vec<String>) -> Result<T,String>;
}


impl<T1,T2> FromPlutusData<(T1,T2)> for (T1,T2) where T1: FromPlutusData<T1>, T2: FromPlutusData<T2> {
    fn from_plutus_data(x:PlutusData,attribs:&Vec<String>) -> Result<(T1,T2),String> {
        match x.as_list()  {
            Some(p) if p.len() == 2 => {
                let key_a_plutus_item = p.get(0);
                let key_b_plutus_item = p.get(1);
                Ok((T1::from_plutus_data(key_a_plutus_item,&attribs)?,
                    T2::from_plutus_data(key_b_plutus_item,&attribs)?
                ))
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
                        Ok((T1::from_plutus_data(key_a_plutus_item,attribs)?,
                            T2::from_plutus_data(key_b_plutus_item,attribs)?
                        ))
                    },
                    None => Err(format!("invalid tuple data.. {:?}: {:?}",x.kind(),x))
                }
            }
        }
    }
}


impl<T1,T2> ToPlutusData for (T1,T2) where T1: ToPlutusData , T2: ToPlutusData {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        let k = self.0.to_plutus_data(&attribs)?;
        let v = self.1.to_plutus_data(&attribs)?;
        let mut pl = cardano_multiplatform_lib::plutus::PlutusList::new();
        pl.add(&k);
        pl.add(&v);
        let cs = cardano_multiplatform_lib::plutus::ConstrPlutusData::new(
            &cardano_multiplatform_lib::ledger::common::value::BigNum::zero(), &pl);
        Ok(cardano_multiplatform_lib::plutus::PlutusData::new_constr_plutus_data(&cs))
    }
}


impl<K : ToPlutusData + Clone,V : ToPlutusData + Clone> ToPlutusData for HashMap<K,V> {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        let mut map = cardano_multiplatform_lib::plutus::PlutusMap::new();
        for kvp in self.iter() {
            let encoded_k = kvp.0.clone().to_plutus_data(&attribs);
            let encoded_v = kvp.1.clone().to_plutus_data(&attribs);
            map.insert(&encoded_k?,&encoded_v?);
        }
        Ok(cardano_multiplatform_lib::plutus::PlutusData::new_map(&map))
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Option<T>> for Option<T> {
    fn from_plutus_data(x:PlutusData,attribs:&Vec<String>) -> Result<Option<T>,String> {
        let result = T::from_plutus_data(x,attribs);
        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(format!("Failed to unpack option value from plutus data! Error: {}",e))
        }
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Box<T>> for Box<T> {
    fn from_plutus_data(x:PlutusData,attribs:&Vec<String>) -> Result<Box<T>,String> {
        let result = T::from_plutus_data(x,attribs);
        match result {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(format!("Failed to unpack option value from plutus data! Error: {}",e))
        }
    }
}

impl<T : ToPlutusData> ToPlutusData for Vec<T> {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        let mut vec_items = cardano_multiplatform_lib::plutus::PlutusList::new();
        for yyy in self {
            let vx = yyy.to_plutus_data(&attribs);
            vec_items.add(&vx?);
        }
        Ok(
            cardano_multiplatform_lib::plutus::PlutusData::new_list(&vec_items)
        )
    }
}

impl<T1 :std::hash::Hash + std::cmp::Eq + FromPlutusData<T1>,T2 : FromPlutusData<T2>> FromPlutusData<HashMap<T1,T2>> for HashMap<T1,T2> {
    fn from_plutus_data(p:PlutusData,attribs:&Vec<String>) -> Result<HashMap<T1,T2>,String> {
        
        match p.as_map() {
            None => Err(format!("Attempting to decode a hashmap but instead found: {:?}.",p.kind())),
            Some(m) => {
                let items = m.keys();
                let mut result = HashMap::new();
                for n in 0 .. items.len() {
                    let the_key = items.get(n);
                    let k = T1::from_plutus_data(the_key.clone(),attribs);
                    let the_value = m.get(&the_key).map_or(Err(String::from("found null value in plutus data. not supported")),|x|Ok(x))?;
                    let v = T2::from_plutus_data(the_value,attribs);
                    result.insert(k?,v?);
                }
                Ok(result)
            }
        }
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Vec<T>> for Vec<T> {
    fn from_plutus_data(p:PlutusData,attribs:&Vec<String>) -> Result<Vec<T>,String> {
        match p.as_list() {
            Some(pl) => {{
                let mut result : Vec<T> = vec![];
                for xi in 0 .. pl.len() {
                    match T::from_plutus_data(pl.get(xi),attribs) {
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
    fn from_plutus_data(x:PlutusData,attribs:&Vec<String>) -> Result<String,String> {
        let b16 : bool = attribs.iter().any(|a|a.to_lowercase() == "base_16");
        match x.as_bytes() {
            Some(bytes) if b16 => {
                Ok(hex::encode(bytes))
            },
            Some(bytes) => {
                match std::str::from_utf8(&bytes) {
                    Ok(s) => Ok(s.to_owned()),
                    Err(e) => Err(format!("{:?}",e))
                }  
            }
            None => Err(format!("expected string bytes, found something else: {:?}..",x.kind()))
        }
    }
}

impl ToPlutusData for String {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        let b16 : bool = attribs.iter().any(|a|a.to_lowercase() == "base_16");
        let bytes = String::as_bytes(self).to_vec();
        if b16 {
            match hex::decode(bytes) {
                Ok(hex_bytes) => Ok(cardano_multiplatform_lib::plutus::PlutusData::new_bytes(hex_bytes)),
                Err(e) => Err(format!("{:?}",e))
            }
        } else {
            Ok(cardano_multiplatform_lib::plutus::PlutusData::new_bytes(bytes))
        }
    }
}

impl<T: ToPlutusData> ToPlutusData for &Option<T> {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        match self {
            Some(v) => v.to_plutus_data(&attribs),
            None => Err(String::from("Not possible to encode None to plutus data.")),
        }
    }
}


impl<T: ToPlutusData> ToPlutusData for Option<T> {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        match self {
            Some(v) => v.to_plutus_data(&attribs),
            None => Err(String::from("Not possible to encode None to plutus data.")),
        }
    }
}

impl<T: ToPlutusData + Clone + ?Sized> ToPlutusData for Box<T> {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        let inner_item : T = Box::into_inner(self.to_owned());
        inner_item.to_plutus_data(&attribs)
    }
}


impl FromPlutusData<bool> for bool {
    fn from_plutus_data(x:PlutusData,attribs:&Vec<String>) -> Result<bool,String> {
        let num_rep : bool = attribs.iter().any(|a|a.to_lowercase() == "repr_bool_as_num");
        if num_rep {
            match x.as_integer() {
                Some(n) if n.to_str() == "0" => Ok(false),
                Some(n) if n.to_str() == "1" => Ok(true),
                _ => {
                    match x.as_constr_plutus_data() {
                        Some(c) if c.alternative().to_str() == "0" && c.data().len() == 0 => {
                            Err(String::from("failed to decode plutus data to bool using integer representation. it does seem to be a valid constr 0 [] (false) item. perhaps you should try without using the 'repr_bool_as_num' attributes?"))
                        },
                        Some(c) if c.alternative().to_str() == "1" && c.data().len() == 0 => {
                            Err(String::from("failed to decode plutus data to bool using integer representation. it does seem to be a valid constr 10 [] (true) item. perhaps you should try without using the 'repr_bool_as_num' attributes?"))
                        }
                        _ => Err(format!("cannot decode bool from {:?}",x))
                    }
                }
            }
        } else {
            match x.as_constr_plutus_data() {
                Some(c) if c.alternative().to_str() == "0" && c.data().len() == 0 => {
                    Ok(false)
                },
                Some(c) if c.alternative().to_str() == "1" && c.data().len() == 0 => {
                    Ok(true)
                },
                _ => {
                    match x.as_integer() {
                        Some(n) if n.to_str() == "0" => Err(String::from("failed to decode plutus data to bool. you could try to mark the field with attribute 'repr_bool_as_num', in which case this would have been 'False'.")),
                        Some(n) if n.to_str() == "1" => Err(String::from("failed to decode plutus data to bool. you could try to mark the field with attribute 'repr_bool_as_num', in which case this would have been 'True'.")),
                        _ => Err(format!("failed to decode this plutus data to bool: {:?}",x)),
                    }
                    
                }
            }
        }
        
    }
}

impl ToPlutusData for bool {
    fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<PlutusData,String> {
        let num_rep : bool = attribs.iter().any(|a|a.to_lowercase() == "repr_bool_as_num");        
        match self {
            true if num_rep => Ok(PlutusData::new_integer(&BigInt::from(1))),
            false if num_rep => Ok(PlutusData::new_integer(&BigInt::from(0))),
            true => Ok(
                PlutusData::new_constr_plutus_data(
                    &ConstrPlutusData::new(
                        &BigNum::from(1),
                        &PlutusList::new()
                    )
                )
            ),
            false => Ok(
                PlutusData::new_constr_plutus_data(
                    &ConstrPlutusData::new(
                        &BigNum::from(0),
                        &PlutusList::new()
                    )
                )
            ),
        }
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
                fn to_plutus_data(&self,_attribs:&Vec<String>) -> Result<PlutusData,String> {
                    match cardano_multiplatform_lib::ledger::common::value::BigInt::from_str(&self.to_string()) {
                        Ok(n) => Ok(cardano_multiplatform_lib::plutus::PlutusData::new_integer(&n)),
                        Err(_) => Err(format!("failed to parse {} to BigInt.",self)),
                    }
                }
            }
            impl ToPlutusData for &$T {
                fn to_plutus_data(&self,_attribs:&Vec<String>) -> Result<PlutusData,String> {
                    match cardano_multiplatform_lib::ledger::common::value::BigInt::from_str(&self.to_string()) {
                        Ok(n) => Ok(cardano_multiplatform_lib::plutus::PlutusData::new_integer(&n)),
                        Err(_) => Err(format!("failed to parse {} to BigInt.",self)),
                    }
                }
            }
            impl FromPlutusData<$T> for $T {
                fn from_plutus_data(p:PlutusData,_attribs:&Vec<String>) -> Result<$T,String> {
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