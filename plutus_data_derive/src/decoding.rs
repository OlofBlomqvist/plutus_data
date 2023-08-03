pub (crate) fn decode_field_value(ty:&syn::Type,attribs:&[String]) -> syn::__private::TokenStream2 {
    
    quote!{|p:plutus_data::PlutusData| -> Result<#ty,String> {
        let mut attributes : Vec<String> = vec![#(String::from(#attribs)),*];
        //println!("Calling {}::from_plutus_data on some data..",stringify!(#ty));
        <#ty>::from_plutus_data(p,&attributes)
    }}}

pub (crate) fn handle_struct_decoding(mut fields:syn::punctuated::Punctuated<syn::Field,syn::token::Comma>,name:syn::Ident,attributes:Vec<syn::Attribute>) -> proc_macro::TokenStream {
    
    let name_string = name.to_string();
    let field_count = fields.len();
    if field_count == 0 {
        panic!("Cannot create decoder for structs with no fields. {:?}",name.to_string())
    }

    let use_unnamed = fields.first().unwrap().ident.is_none();
    let mut extracted_values : Vec<_> = vec![];
    let struct_attribs = attributes.into_iter().map(|a| a.path.get_ident().unwrap().to_string() )
        .collect::<Vec<String>>();
    
    for (i,f) in fields.iter_mut().enumerate() {
        let field_attr_iter = f.clone().attrs.into_iter().map(|a| a.path.get_ident().unwrap().to_string() );
        
        let mut attribs = field_attr_iter.clone().collect::<Vec<String>>();
        for x in struct_attribs.iter() {attribs.push(x.to_owned())};
        let getter = decode_field_value(&f.ty,&attribs);
        extracted_values.push(match &f.ident {
            Some(fident) => quote! { #fident : {(#getter)(items[#i].clone())?} },
            None => quote!{ (#getter)({items[#i].clone()})? }
        });
    };

    let creator = 
        if field_count == 0 {
            panic!("Cannot create decoder for struct with no fields..")
        } else if use_unnamed {
            quote!{ 
                {if ilen < #field_count {
                    return Err(format!("Invalid number of (unnamed) items in the plutus data. Found: {} , Expected: {}..expected struct type name: {}",ilen,#field_count,#name_string))
                } else {
                    #name(#(#extracted_values),*) 
                }}
            }
        } else {
            quote!{  
                {if ilen < #field_count {
                    return Err(format!("Invalid number of items in the plutus data. Found: {} , Expected: {}.. expected struct type name: {}",
                        ilen,#field_count,#name_string))
                } else {
                    #name { #(#extracted_values),* }
                }}
                
            }
        };
    
    
    let result = quote!{
        impl plutus_data::FromPlutusData<#name> for #name {
            fn from_plutus_data(x:plutus_data::PlutusData,_attribs:&[String]) -> Result<#name,String> {
                //println!("from_plutus_data (struct) was called for type {}",#name_string);
                let name_str = #name_string;
               match x {
                    PlutusData::Constr(cc) => {
                        let items = cc.fields;
                        let ilen = items.len();
                        //println!("We have now entered a constr with {} items. ({})",ilen, stringify!(#name));
                        let result = Ok(#creator);
                        result
                    },
                    _ => Err(format!("plutus_data macro ran in to an error while attempting to decode a struct of type {}: Not valid struct data. Expected constr data but found: {:?} --> {:?}",name_str,x,plutus_data::to_hex(&x)))
                }
            }
        }
    };
    
    proc_macro::TokenStream::from(result)

}



pub (crate) fn data_enum_decoding_handling(v:syn::DataEnum,name:syn::Ident,attributes:Vec<syn::Attribute>) -> proc_macro::TokenStream {
    
    let name_string = name.to_string();
    
    let mut constructor_cases = vec![];
    let enum_attribs = attributes.into_iter().map(|a| a.path.get_ident().unwrap().to_string() )
        .collect::<Vec<String>>();

    if enum_attribs.iter().any(|a|a.to_lowercase() == "ignore_option_container") {
        panic!("enums should not be marked with ignore_option_container. just mark whatever field contains your option item.")
    }

    
    for (i,v) in v.variants.iter().enumerate() {
        
        let u64i = i as u64;
        let variant_ident = &v.ident;
        let variant_fields = v.fields.clone();
        let field_count = v.fields.len();

        let is_forced = v.clone().attrs.into_iter().any(|a|format!("{:?}",a.path.get_ident().unwrap().to_string()).contains("force_variant"));
        
        if is_forced {
            crate::info(&name,"When decoding items that contain this enum, we will assume the plutus data only contains the data of the forced variant, and not the wrapping emum.");
        }

        if field_count == 0 {
            constructor_cases.push(quote! {
                #u64i => { Ok(#name::#variant_ident) }
            });
            continue;
        }
        let mut cloned_fields = variant_fields.clone();
        
        let mut extracted_values : Vec<_> = vec![];

        let variant_attribs = v.clone().attrs.into_iter()
            .map(|a| a.path.get_ident().unwrap().to_string() )
            .collect::<Vec<String>>();



        for (ii,f) in cloned_fields.iter_mut().enumerate() {

            let mut attribs = 
                f.clone().attrs.into_iter()
                        .map(|a| a.path.get_ident().unwrap().to_string() )
                        .collect::<Vec<String>>();
            
         
            for x in variant_attribs.iter() {attribs.push(x.to_owned())};

           
            for x in enum_attribs.clone() {attribs.push(x)};

            
            let getter = decode_field_value(&f.ty,&attribs);
            
            if is_forced {
                extracted_values.push(quote! { (#getter)(selfie)? });
            } else {
                extracted_values.push(match &f.ident {
                    Some(fident) => {
                        // quote! {
                        //     #fident : (#getter)({println!("reading named field value: {}",stringify!(#fident));items.get(#ii)})?
                        // }
                        quote! {
                            #fident : (#getter)({items[#ii].clone()})?
                        }
                    },
                    None => 
                        // quote!{ {
                        //     (#getter)({println!("reading unnamed field number {} value",#ii);items.get(#ii)})?
                        // }
                        quote!{ {
                            (#getter)({items[#ii].clone()})?
                        }
                    }
                });
           }
        }

        let use_unnamed = v.fields.clone().iter().next().unwrap().ident.is_none();
        //let strident = format!("{}::{}",name.to_string(),variant_ident.to_string());
        let variant_constructor = {
            if field_count == 0 {
                panic!("Cannot create decoder for struct with no fields..")
            } else if use_unnamed {
                //quote!{ {{println!("using unnamed field for {}",#strident);Ok(#name::#variant_ident(#(#extracted_values),*)) }}}
                quote!{ {{Ok(#name::#variant_ident(#(#extracted_values),*)) }}}
            } else {
                //quote!{ {{println!("Using named field for {}",#strident);Ok(#name::#variant_ident { #(#extracted_values),* }) }}}
                quote!{ {{Ok(#name::#variant_ident { #(#extracted_values),* }) }}}
            }
        };
        
        if is_forced {
            return proc_macro::TokenStream::from(quote! {
                impl plutus_data::FromPlutusData<#name> for #name {
                    fn from_plutus_data(selfie:plutus_data::PlutusData,_attribs:&[String]) -> Result<#name,String> {
                        let name_str = #name_string;
                        //println!("(forced variant) from_plutus_data (enum) was called for type {}",name_str);
                        #variant_constructor
                    }
                }
            });
        }
        
        constructor_cases.push(quote! { #u64i => { 
            //println!("Using constructor {}",#u64i);
            #variant_constructor 
        } });

    };

    let name_string = name.to_string();

    proc_macro::TokenStream::from(quote!{
        use plutus_data::*;
        impl plutus_data::FromPlutusData<#name> for #name {
            fn from_plutus_data(x:plutus_data::PlutusData,attribs:&[String]) -> Result<#name,String> {
                let name_str = #name_string;
                //println!("from_plutus_data (enum) was called for type {} -- {:?}",name_str,&x);
                match x {
                    PlutusData::Constr(cc) => {
                        //println!("{cc:?}");

                        let constructor_u64 = match cc.tag {
                            121..=127 => cc.tag - 121,
                            1280..=1400 => cc.tag - 1280 + 7,
                            102 => if let Some(xq) = cc.any_constructor { xq } else {
                                return  Err(format!("constructor 102 was not expected"))
                            },
                            xxx => return  Err(format!("Unexpected constructor for {:?}:{:?} ",name_str,xxx))
                        };

                        //println!("constr: {:?}",constructor_u64);

                        let mut items = cc.fields;
                        //println!("FOUND {} ITEMS!",items.len());
                        let result = match constructor_u64 {
                            #(#constructor_cases),*
                            i => Err(format!("Unexpected constructor for {}: {}...",name_str,i))
                        };
                        result
                    },
                    _ => Err(format!("This is not a valid constr data item.. cannot decode into enum.. actual type: {:?}.. ::{:?}:: ==> {:?}",x,name_str,x))
                }
            }
        }
    })
}