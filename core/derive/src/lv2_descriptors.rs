use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Result, Token, Type};

/// An instance descriptor that should be exported.
///
/// The instance descriptor is defined by the plugin type.
struct Lv2InstanceDescriptor {
    plugin_type: Type,
}

impl Parse for Lv2InstanceDescriptor {
    fn parse(input: ParseStream) -> Result<Self> {
        let plugin_type = input.parse()?;
        Ok(Lv2InstanceDescriptor { plugin_type })
    }
}

impl Lv2InstanceDescriptor {
    /// Create a matching arm for the plugin.
    ///
    /// The root function receives an index and has to return one plugin descriptor per index,
    /// or NULL. In this crate's implementation, this index is matched in a `match` statement and
    /// this method creates a match arm for this plugin.
    fn make_index_match_arm(&self, index: u32) -> impl ::quote::ToTokens {
        let plugin_type = &self.plugin_type;
        quote! {
            #index => &<#plugin_type as PluginInstanceDescriptor>::DESCRIPTOR,
        }
    }
}

/// A collection for instance descriptors.
///
/// The contained instance descriptors are used to create the export function `lv2_descriptor` that
/// tells the host of a library's plugins.
struct Lv2InstanceDescriptorList {
    descriptors: Punctuated<Lv2InstanceDescriptor, Token![,]>,
}

impl Parse for Lv2InstanceDescriptorList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            descriptors: Punctuated::parse_terminated(input)?,
        })
    }
}

impl Lv2InstanceDescriptorList {
    /// Create the `lv2_descriptor` function.
    ///
    /// This function tells the host of a library's plugin instances by returning one plugin
    /// instance per index.
    fn make_descriptor_function(&self) -> impl ::quote::ToTokens {
        let index_matchers = self
            .descriptors
            .iter()
            .enumerate()
            .map(|(i, desc)| desc.make_index_match_arm(i as u32));

        quote! {
            /// Return a raw pointer to the plugin descriptor with the given index.
            ///
            /// This function is used by the host to discover plugins in the library. The host calls it with an ascending index and stores every returned descriptor,
            /// until a null pointer is returned.
            ///
            /// # Safety
            ///
            /// This function is primarily unsafe because it's a method that's directly called by the host. It doesn't actually do anything that unsafe.
            ///
            /// The returned pointer references a constant and there is valid as long as the library is loaded.
            #[no_mangle]
            pub unsafe extern "C" fn lv2_descriptor(index: u32) -> *const LV2_Descriptor {
                match index {
                    #(#index_matchers)*
                    _ => ::std::ptr::null()
                }
            }
        }
    }
}

/// Generate external symbols for LV2 plugins.
#[inline]
pub fn lv2_descriptors_impl(input: TokenStream) -> TokenStream {
    let list: Lv2InstanceDescriptorList = parse_macro_input!(input);
    let export_function = list.make_descriptor_function();

    (quote! {
        #export_function
    })
    .into()
}
