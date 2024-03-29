
#![feature(proc_macro_diagnostic)]
use quote::*;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields,FieldsNamed,FieldsUnnamed};

#[macro_use] extern crate quote;
extern crate syn;

#[allow(dead_code)]
pub (crate) fn emit_msg(ident_span:&syn::Ident,x:&str,lvl:proc_macro::Level) {
    proc_macro::Diagnostic::spanned(
        syn::spanned::Spanned::span(ident_span).unwrap(), 
        lvl, 
        x                 
    ).emit();
}

#[allow(dead_code)]
pub (crate) fn info(_ident_span:&syn::Ident,_x:&str) {
    //emit_msg(ident_span,x,proc_macro::Level::Note);
}

#[allow(dead_code)]
pub (crate) fn err(ident_span:&syn::Ident,x:&str) {
    emit_msg(ident_span,x,proc_macro::Level::Error);
}

#[allow(dead_code)]
pub (crate) fn warn(ident_span:&syn::Ident,x:&str) {
    emit_msg(ident_span,x,proc_macro::Level::Warning);
}

mod encoding;
mod decoding;
use encoding::*;
use decoding::*;


#[proc_macro_derive(ToPlutusDataDerive,attributes(
    base_16,
    force_variant,repr_bool_as_num,
    ignore_option_container,
    debug_re_encoding
))]
pub fn to_plutus_data_macro(input: TokenStream) -> TokenStream {

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    
    match input.data {

        syn::Data::Union(_) => {
            err(&name,"Cannot use union types with the plutus_data crate macros.");
            let hmm = quote! {};
            TokenStream::from(hmm)
        }

        Data::Struct(DataStruct { fields: Fields::Unit, .. }) => {
            err(&name,"Cannot contain fields of type unit.");
            let hmm = quote! {};
            TokenStream::from(hmm)
        },

        Data::Struct(DataStruct {fields: Fields::Unnamed(FieldsUnnamed { 
            unnamed: unnamed_fields,..
        }),..}) => handle_struct_encoding(unnamed_fields,name,input.attrs),
        
        Data::Struct(DataStruct {fields: Fields::Named(FieldsNamed { 
            named:named_fields, .. 
        }), .. }) => handle_struct_encoding(named_fields,name,input.attrs),

        syn::Data::Enum(ve) => 
            data_enum_encoding_handling(ve,name,input.attrs)
    }
}




#[proc_macro_derive(FromPlutusDataDerive,attributes(base_16,force_variant,repr_bool_as_num,ignore_option_container))]
pub fn from_plutus_data_macro(input: TokenStream) -> TokenStream {

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    
    match input.data {

        syn::Data::Union(_) => {
            err(&name,"Cannot use union types with the plutus_data crate macros.");
            TokenStream::from(quote! {})
        }

        Data::Struct(DataStruct { fields: Fields::Unit, .. }) => {
            err(&name,"Cannot contain fields of type unit.");
            TokenStream::from(quote! {})
        },

        Data::Struct(DataStruct {fields: Fields::Unnamed(FieldsUnnamed { 
            unnamed: unnamed_fields,..
        }),..}) => handle_struct_decoding(unnamed_fields,name,input.attrs),
        
        Data::Struct(DataStruct {fields: Fields::Named(FieldsNamed { 
            named:named_fields, .. 
        }), .. }) => handle_struct_decoding(named_fields,name,input.attrs),

        syn::Data::Enum(vf) => 
            data_enum_decoding_handling(vf,name,input.attrs)
    }
}
