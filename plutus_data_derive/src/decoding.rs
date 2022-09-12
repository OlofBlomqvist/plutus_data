pub (crate) fn decode_field_value(ty:&syn::Type,attribs:&Vec<String>) -> syn::__private::TokenStream2 {
    
    quote!{|p:plutus_data::PlutusData| -> Result<#ty,String> {
        let mut attributes : Vec<String> = vec![#(String::from(#attribs)),*];
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
            Some(fident) => quote! { #fident : (#getter)(items.get(#i))? },
            None => quote!{ (#getter)(items.get(#i))? }
        });
    };

    let creator = 
        if field_count == 0 {
            panic!("Cannot create decoder for struct with no fields..")
        } else {
            if use_unnamed {
                quote!{ 
                    {if ilen < #field_count {
                        return Err(format!("Invalid number of (unnamed) items in the plutus data. Found: {} , Expected: {}.. ",ilen,#field_count))
                    } else {
                        #name(#(#extracted_values),*) 
                    }}
                }
            } else {
                quote!{  
                    {if ilen < #field_count {
                        return Err(format!("Invalid number of items in the plutus data. Found: {} , Expected: {}..",ilen,#field_count))
                    } else {
                        #name { #(#extracted_values),* }
                    }}
                    
                }
            }
        };
    
    
    let result = quote!{
        impl plutus_data::FromPlutusData<#name> for #name {
            fn from_plutus_data(x:plutus_data::PlutusData,attribs:&Vec<String>) -> Result<#name,String> {
                let name_str = #name_string;
                match x.as_constr_plutus_data() {
                    Some(mut cc) => {
                        let items = cc.data();
                        let ilen = items.len();
                        let result = Ok(#creator);
                        result
                    },
                    None => Err(format!("Not valid struct data. Expected constr data but found: {:?}",x.kind()))
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
    for (i,v) in v.variants.iter().enumerate() {
        
        let u64i = i as u64;
        let variant_ident = &v.ident;
        let variant_fields = v.fields.clone();
        let field_count = v.fields.len();

        let is_forced = v.clone().attrs.into_iter().any(|a|format!("{:?}",a).contains("force_variant"));
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
                    Some(fident) => quote! {#fident : (#getter)(items.get(#ii))?},
                    None => quote!{ (#getter)(items.get(#ii))? }
                });
           }
        }

        let use_unnamed = v.fields.clone().iter().next().unwrap().ident.is_none();

        let variant_constructor = {
            if field_count == 0 {
                panic!("Cannot create decoder for struct with no fields..")
            } else {
                if use_unnamed {
                    quote!{ Ok(#name::#variant_ident(#(#extracted_values),*)) }
                } else {
                    quote!{ Ok(#name::#variant_ident { #(#extracted_values),* }) }
                }
            }
        };
        
        if is_forced {
            return proc_macro::TokenStream::from(quote! {
                impl plutus_data::FromPlutusData<#name> for #name {
                    fn from_plutus_data(selfie:plutus_data::PlutusData,attribs:&Vec<String>) -> Result<#name,String> {
                        let name_str = #name_string;
                        #variant_constructor
                    }
                }
            });
        }

        constructor_cases.push(quote! { #u64i => { #variant_constructor } });

    };

    let name_string = name.to_string();

    proc_macro::TokenStream::from(quote!{
        use plutus_data::*;
        impl plutus_data::FromPlutusData<#name> for #name {
            fn from_plutus_data(x:plutus_data::PlutusData,attribs:&Vec<String>) -> Result<#name,String> {
                let name_str = #name_string;
                match x.as_constr_plutus_data() {
                    Some(c) => {
                        let constructor = c.alternative();
                        let constructor_u64 = plutus_data::from_bignum(&constructor);
                        let mut items = c.data();
                        let result = match constructor_u64 {
                            #(#constructor_cases),*
                            i => Err(format!("Unexpected constructor: {}",i))
                        };
                        result
                    },
                    None => Err(format!("This is not a valid constr data item.. cannot decode into enum.. actual type: {:?}.. ::{:?}::",x.kind(),name_str))
                }
            }
        }
    })
}