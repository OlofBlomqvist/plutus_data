use super::*;

pub (crate) fn encode_field_value(field:&syn::Field,selfie:bool,attribs:&Vec<String>) -> syn::__private::TokenStream2 {

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

    quote! {
        let attributes : Vec<String> = vec![#(String::from(#attribs)),*];
        #ident_ref.to_plutus_data(&attributes)
    }
}



pub (crate) fn handle_struct_encoding(
    mut fields:syn::punctuated::Punctuated<syn::Field,syn::token::Comma>,
    name:syn::Ident,attributes:Vec<syn::Attribute>
) -> proc_macro::TokenStream {
    let attributes = attributes.into_iter().map(|a| a.path.get_ident().unwrap().to_string() )
                            .collect::<Vec<String>>();
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
        let mut attribs = 
            my_field.clone().attrs.into_iter()
                .map(|a| a.path.get_ident().unwrap().to_string() )
                .collect::<Vec<String>>();

        for x in &attributes {
            attribs.push(x.clone())
        }

        let val_quote = encode_field_value(&my_field,field_idents.is_empty(),&attribs);
        encoder_field_handlers.push(quote!{
            let vg : Result<plutus_data::PlutusData,String> = {
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
    
    let combo = quote! {
        use plutus_data::*;
        impl plutus_data::ToPlutusData for #name {
            fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<plutus_data::PlutusData,String> {
                #woop
                let mut items = plutus_data::PlutusList::new();
                #(#encoder_field_handlers)*
                let bzero = plutus_data::convert_to_big_num(&0);
                Ok( plutus_data::PlutusData::new_constr_plutus_data(
                        &plutus_data::ConstrPlutusData::new(&bzero,&items)
                ) )
            }
            
        }
    };
    TokenStream::from(combo)
}

pub (crate) fn data_enum_encoding_handling(v:syn::DataEnum,name:syn::Ident,attributes:Vec<syn::Attribute>) -> proc_macro::TokenStream {
    
    let variants = v.variants.into_iter();
    let mut encoder_variant_handlers = vec![];
    let attributes = attributes.into_iter().map(|a| a.path.get_ident().unwrap().to_string() )
                            .collect::<Vec<String>>();
    let alphabet = 
        (b'a'..=b'z')
        .map(|c| c as char)
        .filter(|c| c.is_alphabetic())
        .collect::<Vec<_>>();

    let mut constructor_id : i64 = -1;

    for ev in variants {

        constructor_id = constructor_id + 1;
        let variant_ident = ev.ident.clone();        
        let field_count = ev.fields.len();
        let variant_attribs = ev.clone().attrs.into_iter()
            .map(|a| a.path.get_ident().unwrap().to_string() )
            .collect::<Vec<String>>();
        let is_forced = ev.clone().attrs.into_iter().any(|a|format!("{:?}",a.path.get_ident().unwrap().to_string()).contains("force_variant"));
       
        if is_forced {
            crate::info(&variant_ident,&format!("When encoding this type, we will only allow it to be of the specified variant '{}', and it will be packed as the inner data of the variant, ie. it will not be wrapped in constr data.",variant_ident.to_string()));
        }

        if field_count == 0 {

            if is_forced {
                encoder_variant_handlers.clear();
            }

            encoder_variant_handlers.push(quote!{
                #name::#variant_ident => {
                    let my_constructor_id = #constructor_id;
                    let big_num = plutus_data::convert_to_big_num(&my_constructor_id);
                    let items = plutus_data::PlutusList::new();
                    let item = plutus_data::ConstrPlutusData::new(&big_num,&items);
                    Ok(plutus_data::PlutusData::new_constr_plutus_data(&item))
                }
            });

            if is_forced { 
                let enum_name = name.to_string();
                let variant_name = variant_ident.to_string();
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
            
            let mut attribs = 
                field.clone().attrs.into_iter()
                    .map(|a| a.path.get_ident().unwrap().to_string() )
                    .collect::<Vec<String>>();
            
            for x in &attributes {
                attribs.push(x.clone());
            }

            for x in &variant_attribs {
                attribs.push(x.clone());
            }

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
                    

                            
                    let val_quote = encode_field_value(&named_field_for_an_ident,false,&attribs);
                    encoder_field_handlers.push(quote!{
                        let v : Result<plutus_data::PlutusData,String> = {
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
                    
                    let val_quote = encode_field_value(&field_ident_field,false,&attribs);
                    encoder_field_handlers.push(quote!{
                        let v : Result<plutus_data::PlutusData,String> = {
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
                    #name::#variant_ident #varfieldrefs => {
                        let mut items = plutus_data::PlutusList::new();
                        #(#encoder_field_handlers);*
                        let result = items.get(0);
                        Ok(result)
                    }
                });
            } else {
                
                encoder_variant_handlers.push(quote!{
                    #name::#variant_ident #varfieldrefs => {
                        let my_constructor_id = #constructor_id;
                        let big_num = plutus_data::convert_to_big_num(&my_constructor_id);
                        let mut items = plutus_data::PlutusList::new();
                        #(#encoder_field_handlers);*
                        let item = plutus_data::ConstrPlutusData::new(&big_num,&items);
                        Ok(plutus_data::PlutusData::new_constr_plutus_data(&item))
                    }
                });
            }
        } else {
                
            
        encoder_variant_handlers.push(quote!{
                #name::#variant_ident #varfieldrefs => {
                    //println!("not ignoring container on item: {}",stringify!(#variant_ident));
                    let my_constructor_id = #constructor_id;
                    let big_num = plutus_data::convert_to_big_num(&my_constructor_id);
                    let mut items = plutus_data::PlutusList::new();
                    #(#encoder_field_handlers);*
                    let item = plutus_data::ConstrPlutusData::new(&big_num,&items);
                    Ok(plutus_data::PlutusData::new_constr_plutus_data(&item))
                }
            });
        }

        if is_forced { 
            let enum_name = name.to_string();
            let variant_name = variant_ident.to_string();
            let variant_full_name = format!("{}::{}",enum_name,variant_name);
            encoder_variant_handlers.push(quote!{
                _ => Err(format!("This enum has been marked to only allow a specific variant ({:?}) to be used with plutus encoding.",#variant_full_name))
            });
            break 
        }
        
    }

    let combo = quote! {
        impl plutus_data::ToPlutusData for #name {
            fn to_plutus_data(&self,attribs:&Vec<String>) -> Result<plutus_data::PlutusData,String> {
                match self {
                    #(#encoder_variant_handlers),*
                }
            }
        }
    };

    TokenStream::from(combo)
}



