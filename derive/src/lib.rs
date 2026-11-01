use quote::quote;
use syn::{DataStruct, DeriveInput, parse_macro_input};

#[proc_macro_derive(Component, attributes(caldata))]
pub fn derive_xml_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data.clone() {
        syn::Data::Struct(s) => ComponentStruct::parse(input, s).impl_component(),
        syn::Data::Enum(e) => panic!("Enum not supported"),
        syn::Data::Union(_) => panic!("Union not supported"),
    }
    .into()
}

struct ComponentStruct {
    ident: syn::Ident,
    generics: syn::Generics,
}

impl ComponentStruct {
    pub fn parse(input: DeriveInput, data: DataStruct) -> Self {
        Self {
            ident: input.ident,
            generics: input.generics,
        }
    }

    pub fn impl_component(&self) -> proc_macro2::TokenStream {
        quote! {}
    }
}
