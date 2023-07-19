use crate::ToPlutusData;
use pallas_primitives::babbage::*;


pub struct CustomPlutus(pallas_primitives::babbage::PlutusData);

impl CustomPlutus {
        
    pub fn make_map<
        K : ToPlutusData,
        V : ToPlutusData
    >(original_map:&std::collections::HashMap<K,V>, attributes:&[String]) -> Result<CustomPlutus,String> {
        
        let mut items = vec![];

        for kvp in original_map.iter() {
            let encoded_k = kvp.0.to_plutus_data(attributes)?;
            let encoded_v = kvp.1.to_plutus_data(attributes)?;
            items.push((encoded_k,encoded_v));
        }

        Ok(CustomPlutus(PlutusData::Map(
            pallas_codec::utils::KeyValuePairs::Def(
                items
            )
        )))
    }

    pub fn to_hex(&self) -> String {
        use pallas_primitives::Fragment;
        hex::encode(self.0.encode_fragment().unwrap())
    }
    
    pub fn as_pallas(&self) -> Option<&pallas_primitives::babbage::PlutusData> {
        Some(&self.0)
    }

    // pub fn as_raw_bytes(&self) -> Option<&Vec<u8>> {
    //     match self {
    //         CustomPlutus::RawBytes(b) => Some(b),
    //         CustomPlutus::Pallas(_) => None,
    //     }
    // }

    pub fn p_str(s:&str) -> CustomPlutus {
        let bytes = s.as_bytes().to_vec();
        Self(pallas_primitives::alonzo::PlutusData::BoundedBytes(bytes.into()))
    }
    
    pub fn to_big_int(n:i64) -> pallas_primitives::alonzo::BigInt {
        pallas_primitives::alonzo::BigInt::Int(pallas_codec::utils::Int::from(n))
    }

    pub fn to_big_uint(n:u64) -> pallas_primitives::alonzo::BigInt {
        pallas_primitives::alonzo::BigInt::Int(pallas_codec::utils::Int::from(n as i64))
    }

    pub fn big_int(n:i64) -> Self {
        Self(PlutusData::BigInt(pallas_primitives::alonzo::BigInt::Int(pallas_codec::utils::Int::from(n))))
    }
    
    pub(crate) fn make_tup(tag:u64,key:pallas_primitives::babbage::PlutusData,value:pallas_primitives::babbage::PlutusData) -> pallas_primitives::babbage::PlutusData {
        pallas_primitives::babbage::PlutusData::Constr(pallas_primitives::babbage::Constr { 
            tag: tag + 121, 
            any_constructor:  Some(tag), 
            fields: vec![key,value]
        })
    }
    
    pub fn make_constr(tag:u64,fields:Vec<pallas_primitives::babbage::PlutusData>) -> pallas_primitives::babbage::PlutusData {
        pallas_primitives::babbage::PlutusData::Constr(pallas_primitives::babbage::Constr { 
            tag: 121 + tag,
            any_constructor:  Some(tag), 
            fields
        })
    }
     
    pub fn make_list<
        T : ToPlutusData + std::fmt::Debug
    >(items:&Vec<T>, attributes:&[String]) -> Result<CustomPlutus,String> {
        
        let mut encoded_items = vec![];
        
        for yyy in items {
            let vx = yyy.to_plutus_data(attributes);
            encoded_items.push(vx?);
        }
        let x = PlutusData::Array(encoded_items);
        Ok(
            Self(x)
        )
    }
    

}