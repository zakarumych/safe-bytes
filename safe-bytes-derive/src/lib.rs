use {proc_macro2::TokenStream, quote::quote, syn::spanned::Spanned as _};

/// Safely implements [`SafeBytes`] via [`PaddingBane`] implementation.
///
/// [`SafeBytes`]: https://docs.rs/safe-bytes/0.1.0/safe_bytes/trait.SafeBytes.html
/// [`PaddingBane`]: https://docs.rs/safe-bytes/0.1.0/safe_bytes/trait.PaddingBane.html
#[proc_macro_derive(SafeBytes)]
pub fn safe_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_safe_bytes(&ast).into()
}

fn impl_safe_bytes(ast: &syn::DeriveInput) -> TokenStream {
    let type_name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(datastruct) => &datastruct.fields,
        _ => panic!("safe_bytes cannot be derived for enums or unions"),
    };

    let field_types = fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>();
    let field_names = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            f.ident
                .clone()
                .unwrap_or_else(|| syn::Ident::new(&format!("_{}", i), ast.span()))
        })
        .collect::<Vec<_>>();

    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        unsafe impl #impl_generics ::safe_bytes::PaddingBane for #type_name #type_generics #where_clause {
            type Fields = (#(::safe_bytes::TypedField<#field_types>,)*);

            #[inline(always)]
            fn get_fields(&self) -> Self::Fields {
                (#(::safe_bytes::typed_field!(*self, #type_name, #field_names),)*)
            }

            #[inline]
            unsafe fn init_padding(fields: Self::Fields, bytes: &mut [::safe_bytes::core::mem::MaybeUninit<u8>]) {
                use {
                    ::safe_bytes::core::{mem::size_of, ptr::write_bytes},
                };

                let (#(#field_names,)*) = fields;
                let mut raw_fields = [#(#field_names.raw,)*];
                raw_fields.sort_unstable_by_key(|f| f.offset);
                let mut offset = 0;
                for field in &raw_fields {
                    if field.offset > offset {
                        let count = field.offset - offset;
                        write_bytes(&mut bytes[offset], 0xfe, count);
                    }
                    offset = field.offset + field.size;
                }

                if size_of::<Self>() > offset {
                    let count = size_of::<Self>() - offset;
                    write_bytes(&mut bytes[offset], 0xfe, count);
                }

                #(
                    let field_bytes = &mut bytes[#field_names.raw.offset .. #field_names.raw.offset + #field_names.raw.size];
                    <#field_types as ::safe_bytes::PaddingBane>::init_padding(#field_names.sub, field_bytes);
                )*
            }
        }
    }
}
