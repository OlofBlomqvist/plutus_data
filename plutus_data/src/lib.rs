#![feature(box_into_inner)]

use std::collections::HashMap;
use std::collections::BTreeMap;

use hex::ToHex;
use pallas_primitives::Fragment;
pub use pallas_primitives::babbage::PlutusData;
pub use plutus_data_derive::FromPlutusDataDerive;
pub use plutus_data_derive::ToPlutusDataDerive;

mod custom_plutus;

pub use custom_plutus::CustomPlutus as cp;
use custom_plutus::*;

pub trait ToPlutusData {
    fn to_plutus_data(&self,attributes:&[String]) -> Result<PlutusData,String>;
}

pub use custom_plutus::CustomPlutus as pd;

pub trait FromPlutusData<T> {
    
    fn from_plutus_data(x:PlutusData,attributes:&[String]) -> Result<T,String>;
}
use std::error::Error;
pub fn from_bytes(x:&[u8]) -> Result<PlutusData, Box<dyn Error>>  {
    PlutusData::decode_fragment(x)
}
pub fn to_bytes(x:&PlutusData) -> Result<Vec<u8>, Box<dyn Error>>  {
    x.encode_fragment()
}
pub fn to_hex(x:&PlutusData) -> Result<String, Box<dyn Error>> {
    let xx = x.encode_fragment()?;
    let hexed : String = xx.encode_hex();
    Ok(hexed)
}
pub fn encode<T : ToPlutusData>(x:&T) -> Result<PlutusData, String>  {
    x.to_plutus_data(&[])
}
pub fn encode_vec<T : ToPlutusData + Clone + std::fmt::Debug>(x:&Vec<T>) -> Result<PlutusData, String>  {
    x.to_plutus_data(&[])
}

impl<T1,T2> FromPlutusData<(T1,T2)> for (T1,T2) where T1: FromPlutusData<T1>, T2: FromPlutusData<T2> {
    fn from_plutus_data(x:PlutusData,attribs:&[String]) -> Result<(T1,T2),String> {
        match x {
            PlutusData::Constr(p) => {
                
                if p.fields.len() != 2 {
                    return Err(format!("expected tuple (list) with two items.. found {:?} with {} items.",p.tag,p.fields.len()))
                }
                
                let key_a_plutus_item = p.fields[0].clone();
                let key_b_plutus_item = p.fields[1].clone();
                Ok((T1::from_plutus_data(key_a_plutus_item,attribs)?,
                    T2::from_plutus_data(key_b_plutus_item,attribs)?
                ))
            },
            PlutusData::Array(p) => {
                if p.len() == 2 {
                    let key_a_plutus_item = p[0].clone();
                    let key_b_plutus_item = p[1].clone();
                    Ok((T1::from_plutus_data(key_a_plutus_item,attribs)?,
                        T2::from_plutus_data(key_b_plutus_item,attribs)?
                    ))
                } else {
                    Err(format!("invalid length for tuple data {p:?}"))
                }
            },
                _ => Err(format!("invalid tuple data {x:?}"))
            }
        }
    }


impl<T1,T2> ToPlutusData for (T1,T2) where T1: ToPlutusData , T2: ToPlutusData {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        let k = self.0.to_plutus_data(attribs)?;
        let v = self.1.to_plutus_data(attribs)?;
        Ok(CustomPlutus::make_tup(k,v))

    }
}


impl<K : ToPlutusData + Clone,V : ToPlutusData + Clone> ToPlutusData for HashMap<K,V> {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        if let Some(p) = CustomPlutus::make_map(self,attribs)?.as_pallas() {
            Ok(p.clone())
        } else {
            Err(String::from("to_plutus_data for hashmap failed."))
        }
    }
}

impl<K : ToPlutusData + Clone,V : ToPlutusData + Clone> ToPlutusData for BTreeMap<K,V> {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        if let Some(p) = CustomPlutus::make_bt_map(self,attribs)?.as_pallas() {
            Ok(p.clone())
        } else {
            Err(String::from("to_plutus_data for btreemap failed."))
        }
    }
}


impl<T : FromPlutusData<T>> FromPlutusData<Box<T>> for Box<T> {
    fn from_plutus_data(x:PlutusData,attribs:&[String]) -> Result<Box<T>,String> {
        let result = T::from_plutus_data(x,attribs);
        match result {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(format!("Failed to unpack option value from plutus data! Error: {}",e))
        }
    }
}

impl<T : ToPlutusData + Clone + std::fmt::Debug> ToPlutusData for Vec<T> {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        if let Some(p) = CustomPlutus::make_list(self,attribs)?.as_pallas() {
            Ok(p.clone())
        } else {
            Err(String::from("to_plutus_data for vec<T> failed."))
        }
    }
}

impl<T1 :std::hash::Hash + std::cmp::Eq + FromPlutusData<T1>,T2 : FromPlutusData<T2>> FromPlutusData<HashMap<T1,T2>> for HashMap<T1,T2> {
    fn from_plutus_data(p:PlutusData,attribs:&[String]) -> Result<HashMap<T1,T2>,String> {
        match p {
            PlutusData::Map(m) => {
                let mut result = HashMap::new();
                for kvp in m.iter() {
                    
                    let the_key = kvp.0.clone();
                    let k = T1::from_plutus_data(the_key.clone(),attribs);

                    let the_val = kvp.1.clone();
                    let v = T2::from_plutus_data(the_val.clone(),attribs);


                    // let the_value = m.get(&the_key).map_or(Err(String::from("found null value in plutus data. not supported")),|x|Ok(x))?;
                    // let v = T2::from_plutus_data(the_value,attribs);

                    result.insert(k?,v?);
                }

                Ok(result)
            },
            _ => Err(format!("Attempting to decode a hashmap but instead found: {:?}.",p))
        }
    }
}


impl<T1 : Ord + std::cmp::Eq + FromPlutusData<T1>,T2 : FromPlutusData<T2>> FromPlutusData<BTreeMap<T1,T2>> for BTreeMap<T1,T2> {
    fn from_plutus_data(p:PlutusData,attribs:&[String]) -> Result<BTreeMap<T1,T2>,String> {
        match p {
            PlutusData::Map(m) => {
                let mut result = BTreeMap::new();
                for kvp in m.iter() {
                    
                    let the_key = kvp.0.clone();
                    let k = T1::from_plutus_data(the_key.clone(),attribs);

                    let the_val = kvp.1.clone();
                    let v = T2::from_plutus_data(the_val.clone(),attribs);

                    result.insert(k?,v?);
                }

                Ok(result)
            },
            _ => Err(format!("Attempting to decode a btreemap but instead found: {:?}.",p))
        }
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Vec<T>> for Vec<T> {
    fn from_plutus_data(p:PlutusData,attribs:&[String]) -> Result<Vec<T>,String> {
        match p {
            PlutusData::Array(pl) => {
                let mut result : Vec<T> = vec![];
                for x in pl {
                    match T::from_plutus_data(x,attribs) {
                        Ok(v) => { result.push(v) },
                        Err(e) => return Err(format!("when decoding a vector, we got this error: {}",e))
                    } 
                }
               
                Ok(result)
            },
            _ => Err(String::from("Failed to decode vec from plutus data because it was not a plutus list."))
        }

    }
}

impl FromPlutusData<String> for String {
    fn from_plutus_data(x:PlutusData,attribs:&[String]) -> Result<String,String> {
        let b16 : bool = attribs.iter().any(|a|a.to_lowercase() == "base_16");
        match x {
            PlutusData::BoundedBytes(bytes) if b16 => {
                Ok(bytes.encode_hex())
            },
            PlutusData::BoundedBytes(bytes) => {
                match std::str::from_utf8(&bytes) {
                    Ok(s) => Ok(s.to_owned()),
                    Err(e) => Err(format!("{:?}",e))
                }  
            },            
            _ => Err(format!("expected string bytes, found something else: {:?}..",x))
        }
    }
}

impl ToPlutusData for String {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        let b16 : bool = attribs.iter().any(|a|a.to_lowercase() == "base_16");
        let bytes = String::as_bytes(self).to_vec();
        if b16 {
            match hex::decode(bytes) {
                Ok(hex_bytes) => Ok(pallas_primitives::alonzo::PlutusData::BoundedBytes(hex_bytes.into())),
                Err(e) => Err(format!("{:?}",e))
            }
        } else {
            Ok(pallas_primitives::alonzo::PlutusData::BoundedBytes(bytes.into()))
        }
    }
}

impl<T: ToPlutusData> ToPlutusData for &Option<T> {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        let ignore_option_container : bool = attribs.iter().any(|a|a.to_lowercase() == "ignore_option_container");
        match self {
            None if ignore_option_container => Err(String::from("Not possible to encode &None to plutus data when using attribute 'ignore_option_container'.")),
            Some(v) if ignore_option_container => v.to_plutus_data(attribs),
            None  => Ok(empty_constr(1)),            
            Some(v)  => {
                Ok(
                    wrap_with_constr(
                        0, 
                        v.to_plutus_data(attribs)?
                    )
                )
            }
        }
    }
}

pub struct ByteVec(Vec<u8>);

impl ToPlutusData for ByteVec {
    fn to_plutus_data(&self,_attributes:&[String]) -> Result<PlutusData,String> {
        Ok(pallas_primitives::alonzo::PlutusData::BoundedBytes(self.0.clone().into()))
    }
}

impl<T : FromPlutusData<T>> FromPlutusData<Option<T>> for Option<T> {
    fn from_plutus_data(x:PlutusData,attribs:&[String]) -> Result<Option<T>,String> {
        let ignore_option_container : bool = attribs.iter().any(|a|a.to_lowercase() == "ignore_option_container");
        
        if ignore_option_container {
            let result = T::from_plutus_data(x,attribs);
            match result {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(format!("Failed to unpack (ignore_option_container) option value from plutus data! Error: {}",e))
            }
        } else {
            match x {
                PlutusData::Constr(c) => {
                    //println!("TAG IS {} AND CONSTR IS {:?}",c.tag,c.any_constructor);
                    
                    let constr = match c.tag {
                        121..=127 => c.tag - 121,
                        1280..=1400 => c.tag - 1280 + 7,
                        102=> if let Some(xq) = c.any_constructor { xq } else {
                            return  Err(format!("constructor 102 was not expected"));
                        },
                        xxx => return  Err(format!("Unexpected constructor {} when decoding an item of type {}",xxx,std::any::type_name::<T>()))
                    };

                    //let constr = if let Some(a) = c.any_constructor { a } else { c.tag - 121} ;
                    match (constr,c.fields.len()) {
                        (0,1) => {
                            Ok(Some(T::from_plutus_data(c.fields[0].clone(),attribs)?))
                        },
                        (1,0) => Ok(None),
                        _ => {
                            Err(String::from("failed to unpack option value. not valid const representation."))
                        }
                    }
                },
                _ => Err(format!("failed to decode option value form plutus data... expected constr, found: {:?}",x)),
            }
        }
    }
}

impl<T: ToPlutusData> ToPlutusData for Option<T> {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        let ignore_option_container : bool = attribs.iter().any(|a|a.to_lowercase() == "ignore_option_container");
        match self {
            None if ignore_option_container => Err(String::from("Not possible to encode None to plutus data when using attribute 'ignore_option_container'.")),
            Some(v) if ignore_option_container => {
                //println!("Encoding without an option container.");
                v.to_plutus_data(attribs)
            },
            None  => Ok(empty_constr(1)),            
            Some(v)  => {
                //println!("Wrapping an item inside of an option constr 0: {:?}",attribs);
                Ok(
                    wrap_with_constr(
                        0, 
                        v.to_plutus_data(attribs)?
                    )
                )
            }
        }
    }
}

impl<T: ToPlutusData + Clone + ?Sized> ToPlutusData for Box<T> {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        let inner_item : T = Box::into_inner(self.to_owned());
        inner_item.to_plutus_data(attribs)
    }
}


impl FromPlutusData<bool> for bool {
    fn from_plutus_data(x:PlutusData,attribs:&[String]) -> Result<bool,String> {
        let num_rep : bool = attribs.iter().any(|a|a.to_lowercase() == "repr_bool_as_num");
        if num_rep {
            match x {
                PlutusData::BigInt(n) => {
                    let s = match n {
                        pallas_primitives::babbage::BigInt::Int(nn) => nn.0.to_string(),
                        BigInt::BigUInt(nn) => nn.to_string(),
                        BigInt::BigNInt(nn) => nn.to_string(),
                    };
                    Ok(s=="1")
                },
                PlutusData::Constr(c) => {
                    Err(format!("cannot decode bool using repr_bool_as_num. it seems to be encoded as a constr, perhaps you did not mean to apply repr_bool_as_num? {:?}",c))
                }
                _ => Err(format!("cannot decode bool with num_rep from {:?}",x))
            }
        } else {
            match x {
                PlutusData::Constr(c) => {
                   
                    let t = match c.tag {
                        121..=127 => c.tag - 121,
                        1280..=1400 => c.tag - 1280 + 7,
                        102 => if let Some(xq) = c.any_constructor { xq } else {
                            return  Err(format!("constructor 102 was not expected"));
                        },
                        xxx => return  Err(format!("Unexpected constructor for {:?}",xxx))
                    };
                    
                    if t == 1 {
                        return Ok(true);
                    }

                    if t == 0 {
                        return Ok(false);
                    }

                    return  Err(format!("Unexpected constructor a boolean! expected 1 or 0 but found: {t}"))
                }
                _ => {
                    match x {
                        PlutusData::BigInt(n) => {
                            Err(format!("failed to decode this plutus data to bool using constr repr. it does seem to be a valid integer encoded bool, perhaps try with using repr_bool_as_num? --> {:?}",n))
                        },
                        _ => Err(format!("cannot decode bool using constr_rep from {:?}",x))
                    }
                    
                }
            }
        }
        
    }
}

impl ToPlutusData for bool {
    fn to_plutus_data(&self,attribs:&[String]) -> Result<PlutusData,String> {
        let num_rep : bool = attribs.iter().any(|a|a.to_lowercase() == "repr_bool_as_num");        
        match self {
            true if num_rep => Ok(CustomPlutus::big_int(1).as_pallas().unwrap().clone()), // todo : fix
            false if num_rep => Ok(CustomPlutus::big_int(0).as_pallas().unwrap().clone()), // todo : fix
            true => Ok(CustomPlutus::make_constr(1, vec![])),
            false => Ok(CustomPlutus::make_constr(0, vec![])),
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

use pallas_primitives::babbage::BigInt;
pub fn convert_to_big_int(i:&i64) -> BigInt { CustomPlutus::to_big_int(*i) }
pub fn convert_u64_to_big_int(i:&u64) -> BigInt { CustomPlutus::to_big_uint(*i) }

mod macros {
    #[macro_export]
    #[doc(hidden)]
    macro_rules! ImplPlutusForNum {
        (@$T:ident) => {
            impl ToPlutusData for $T {
                fn to_plutus_data(&self,_attribs:&[String]) -> Result<PlutusData,String> {
                    
                    match &self.to_string().parse::<i64>() {
                        Ok(n) => Ok(PlutusData::BigInt(pallas_primitives::alonzo::BigInt::Int(pallas_codec::utils::Int::from(*n)))),
                        Err(_) => Err(format!("failed to parse {} to BigInt.",self)),
                    }
                }
            }
            impl ToPlutusData for &$T {
                fn to_plutus_data(&self,_attribs:&[String]) -> Result<PlutusData,String> {
                    match &self.to_string().parse::<i64>() {
                        Ok(n) => Ok(PlutusData::BigInt(pallas_primitives::alonzo::BigInt::Int(pallas_codec::utils::Int::from(*n)))),
                        Err(_) => Err(format!("failed to parse {} to BigInt.",self)),
                    }
                }
            }
            impl FromPlutusData<$T> for $T {
                fn from_plutus_data(p:PlutusData,_attribs:&[String]) -> Result<$T,String> {
                    match p {
                        PlutusData::BigInt(n) => {
                            let s = match n {
                                pallas_primitives::babbage::BigInt::Int(nn) => nn.0.to_string(),
                                pallas_primitives::babbage::BigInt::BigUInt(nn) => nn.to_string(),
                                pallas_primitives::babbage::BigInt::BigNInt(nn) => nn.to_string()
                            };
                            match s.parse::<$T>() {
                                Ok(vc) => Ok(vc),
                                Err(e) => Err(format!("Failed to convert string to number. {}.",e)),
                            }
                        },
                        _ => Err(format!("failed to parse plutus data to num! input kind: {:?} - inner plutus data: {:?}",p,p)),
                    }
                }
            }
        }
    }
}

fn empty_constr(tag_num: u64) -> PlutusData {
    CustomPlutus::make_constr(tag_num, vec![])
}

fn wrap_with_constr(tag_num: u64, data: PlutusData) -> PlutusData {
    CustomPlutus::make_constr(tag_num, vec![data])
}
