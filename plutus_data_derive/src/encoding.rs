use super::*;

pub (crate) fn encode_field_value(field:&syn::Field,selfie:bool,tagged_as_b16:bool) -> syn::__private::TokenStream2 {

    let ident = 
        match &field.ident {
            None => return quote!{Err(String::from("An unknown error occurred while attempting to encode a field value."))},
            Some(x) => x
        };
    
    let ident_ref = if selfie == true { 
        quote! { self.#ident }
    } else {
        quote! { #ident }
    };

    let reffstr = ident_ref.to_string();

    match type_to_str(&field.ty).as_ref() {
        // todo : move this to actual impl?
        "String"|"std::string::String" if tagged_as_b16 => quote!{
            let q = &#ident_ref;
            match hex::decode(q) {
                Ok(hex_bytes) => Ok(cardano_multiplatform_lib::plutus::PlutusData::new_bytes(hex_bytes)),
                Err(e) => Err(
                    format!("a field tagged as base_16 was found to not contain valid hex: {:?}..",
                        #reffstr
                    )
                ) 
            }
        },
        _ => 
            quote!{
                #ident_ref.to_plutus_data()
            }
    }
}



pub (crate) fn handle_struct_encoding(
    mut fields:syn::punctuated::Punctuated<syn::Field,syn::token::Comma>,
    name:syn::Ident
) -> proc_macro::TokenStream {

    let alphabet = 
        (b'a'..=b'z')
        .map(|c| c as char)
        .filter(|c| c.is_alphabetic())
        .collect::<Vec<_>>();
    
    let mut field_idents = vec![];
    let mut encoder_field_handlers = vec![];
    let mut field_iter = fields.iter_mut();
    
    for ii in 0..field_iter.len() {

        let mut field = field_iter.next().unwrap();
        let my_field = match &field.ident {
            Some(_f) => field,
            None => {
                let mut field_ident = quote::format_ident!(
                    "{}", alphabet[ii].to_string()
                );
                field_ident.set_span(name.span());
                field_idents.push(field_ident.clone());
                field.ident = Some(field_ident);
                field
            },
        };

        // todo: this is stupid.
        let tagged_as_bs = 
            my_field.clone().attrs.into_iter()
                .map(|a| a.path.get_ident().unwrap().to_string() )
                .collect::<Vec<String>>()
                .contains(&"base_16".to_owned());

        let val_quote = encode_field_value(&my_field,field_idents.is_empty(),tagged_as_bs);
        encoder_field_handlers.push(quote!{
            let vg : Result<cardano_multiplatform_lib::plutus::PlutusData,String> = {
                #val_quote
            };
            items.add(&vg?);
        });
    }
    

    let woop = 
        if field_idents.is_empty() {
            quote! { }
        } else {
            quote! { let Self (#(#field_idents),*) = self;}
        };
    
    //let (a , b) = (1,2);
    

    let combo = quote! {
        use cardano_multiplatform_lib::*;
        impl plutus_data::ToPlutusData for #name {
            fn to_plutus_data(&self) -> Result<cardano_multiplatform_lib::plutus::PlutusData,String> {
                #woop
                let mut items = cardano_multiplatform_lib::plutus::PlutusList::new();
                #(#encoder_field_handlers)*
                let bzero = cardano_multiplatform_lib::ledger::common::value::BigNum::zero();
                Ok( cardano_multiplatform_lib::plutus::PlutusData::new_constr_plutus_data(
                        &cardano_multiplatform_lib::plutus::ConstrPlutusData::new(&bzero,&items)
                ) )
                //Err(String::from("fake"))
            }
            
        }
    };
    TokenStream::from(combo)
}

pub (crate) fn data_enum_encoding_handling(v:syn::DataEnum,name:syn::Ident) -> proc_macro::TokenStream {
    
    let variants = v.variants.into_iter();
    let mut encoder_variant_handlers = vec![];

    let alphabet = 
        (b'a'..=b'z')
        .map(|c| c as char)
        .filter(|c| c.is_alphabetic())
        .collect::<Vec<_>>();

    let mut constructor_id = -1;

    for ev in variants {
        constructor_id = constructor_id + 1;
        let variant = ev.ident.clone();        
        let field_count = ev.fields.len();
        let istring = constructor_id.clone().to_string();
        let is_forced = ev.attrs.into_iter().any(|a|format!("{:?}",a).contains("force_variant"));
        
        if field_count == 0 {

            if is_forced {
                encoder_variant_handlers.clear();
            }

            encoder_variant_handlers.push(quote!{
                #name::#variant => {
                    let my_constructor_id = #istring;
                    let big_num = cardano_multiplatform_lib::ledger::common::value::BigNum::from_str(&my_constructor_id).unwrap();
                    let items = cardano_multiplatform_lib::plutus::PlutusList::new();
                    let item = cardano_multiplatform_lib::plutus::ConstrPlutusData::new(&big_num,&items);
                    Ok(cardano_multiplatform_lib::plutus::PlutusData::new_constr_plutus_data(&item))
                }
            });

            if is_forced { 
                let enum_name = name.to_string();
                let variant_name = variant.to_string();
                let variant_full_name = format!("{}::{}",enum_name,variant_name);
                encoder_variant_handlers.push(quote!{
                    _ => Err(format!("This enum has been marked to only allow a specific variant ({:?}) to be used with plutus encoding.. but the current value is not of that variant.",#variant_full_name))
                });
                break 
            }

            continue;
        } 

        let mut field_idents = vec![];
        let mut encoder_field_handlers = vec![];
        let mut field_iter = ev.fields.iter();
        let mut named = false;
        for ii in 0..field_count {
            let field = field_iter.next().unwrap();
            match &field.clone().ident {
                Some(ff) => {
                    named = true;
                    let ff_name = ff.clone().to_string();                   
                    let field_ident = 
                        if ff_name.contains("#") {
                            syn::Ident::new_raw(
                                &ff_name.replace("r#",""),
                                ff.span()
                            )
                        } else { 
                            quote::format_ident!("{}",ff_name) 
                        };

                    field_idents.push(field_ident.clone());
                    let named_field_for_an_ident = syn::Field { 
                        attrs: vec![], 
                        vis: syn::Visibility::Inherited,
                        ident: Some(field_ident.clone()), 
                        colon_token: None, 
                        ty: field.ty.clone() 
                    };
                    
                    // todo: this is stupid.
                    let tagged_as_bs = 
                        field.clone().attrs.into_iter()
                            .map(|a| a.path.get_ident().unwrap().to_string() )
                            .collect::<Vec<String>>()
                            .contains(&"base_16".to_owned());
                            
                    let val_quote = encode_field_value(&named_field_for_an_ident,false,tagged_as_bs);
                    encoder_field_handlers.push(quote!{
                        let v : Result<cardano_multiplatform_lib::plutus::PlutusData,String> = {
                            #val_quote
                        };
                        items.add(&v?);
                    }); 
                },

                None => {
                    let f_name = alphabet[ii].to_string();
                    let field_ident = quote::format_ident!("{}",f_name);
                    let field_ident_field = syn::Field { 
                        attrs: vec![], 
                        vis: syn::Visibility::Inherited,
                        ident: Some(field_ident.clone()), 
                        colon_token: None, 
                        ty: field.ty.clone() 
                    };        
                    field_idents.push(field_ident.clone());
                    
                    let val_quote = encode_field_value(&field_ident_field,false,!field.attrs.is_empty());
                    encoder_field_handlers.push(quote!{
                        let v : Result<cardano_multiplatform_lib::plutus::PlutusData,String> = {
                            #val_quote
                        };
                        items.add(&v?);
                    });
                }
            }
        }
        
        let varfieldrefs = if named {
            quote! {{ #(#field_idents),* }}
        } else {
            quote! {( #(#field_idents),* )}
        };

        if is_forced {

            encoder_variant_handlers.clear();
            if field_count == 1 {
                encoder_variant_handlers.push(quote!{
                    #name::#variant #varfieldrefs => {
                        let mut items = cardano_multiplatform_lib::plutus::PlutusList::new();
                        #(#encoder_field_handlers);*
                        let result = items.get(0);
                        Ok(result)
                    }
                });
            } else {
                encoder_variant_handlers.push(quote!{
                    #name::#variant #varfieldrefs => {
                        let my_constructor_id = #istring;
                        let big_num = cardano_multiplatform_lib::ledger::common::value::BigNum::from_str(&my_constructor_id).unwrap();
                        let mut items = cardano_multiplatform_lib::plutus::PlutusList::new();
                        #(#encoder_field_handlers);*
                        let item = cardano_multiplatform_lib::plutus::ConstrPlutusData::new(&big_num,&items);
                        Ok(cardano_multiplatform_lib::plutus::PlutusData::new_constr_plutus_data(&item))
                    }
                });
            }
        } else {
                
            encoder_variant_handlers.push(quote!{
                #name::#variant #varfieldrefs => {
                    let my_constructor_id = #istring;
                    let big_num = cardano_multiplatform_lib::ledger::common::value::BigNum::from_str(&my_constructor_id).unwrap();
                    let mut items = cardano_multiplatform_lib::plutus::PlutusList::new();
                    #(#encoder_field_handlers);*
                    let item = cardano_multiplatform_lib::plutus::ConstrPlutusData::new(&big_num,&items);
                    Ok(cardano_multiplatform_lib::plutus::PlutusData::new_constr_plutus_data(&item))
                }
            });
        }

        if is_forced { 
            let enum_name = name.to_string();
            let variant_name = variant.to_string();
            let variant_full_name = format!("{}::{}",enum_name,variant_name);
            encoder_variant_handlers.push(quote!{
                _ => Err(format!("This enum has been marked to only allow a specific variant ({:?}) to be used with plutus encoding.",#variant_full_name))
            });
            break 
        }
        
    }

    let combo = quote! {
        impl plutus_data::ToPlutusData for #name {
        //impl #name {
            fn to_plutus_data(&self) -> Result<cardano_multiplatform_lib::plutus::PlutusData,String> {
                match self {
                    #(#encoder_variant_handlers),*
                }
            }
        }
    };

    TokenStream::from(combo)
}



