use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, LitStr, Result, Token, Type};

/// A plugin that should be exported with a descriptor.
struct Lv2InstanceDescriptor {
    plugin_type: Type,
    uri: LitStr,
}

impl Parse for Lv2InstanceDescriptor {
    fn parse(input: ParseStream) -> Result<Self> {
        let plugin_type = input.parse()?;
        input.parse::<Token![:]>()?;
        let uri = input.parse()?;
        Ok(Lv2InstanceDescriptor { plugin_type, uri })
    }
}

impl Lv2InstanceDescriptor {
    /// Implement the `PluginInstanceDescriptor` for the plugin.
    ///
    /// By implementing `PluginInstanceDescriptor`, two static objects are created: The URI of the
    /// plugin, stored as a string, and the descriptor, a struct with pointers to the plugin's
    /// basic functions; Like `instantiate` or `run`.
    pub fn make_instance_descriptor_impl(&self) -> impl ::quote::ToTokens {
        let plugin_type = &self.plugin_type;
        let uri = &self.uri;
        quote! {
            unsafe impl ::lv2_core::plugin::PluginInstanceDescriptor for #plugin_type {
                const URI: &'static [u8] = unsafe {
                    union Slices<'a> { str: &'a str, slice: &'a [u8] }
                    Slices { str: concat!(#uri, "\0") }.slice
                };
                const DESCRIPTOR: ::lv2_core::sys::LV2_Descriptor = ::lv2_core::sys::LV2_Descriptor {
                    URI: (&Self::URI[0]) as *const u8 as *const ::std::os::raw::c_char,
                    instantiate: Some(::lv2_core::plugin::PluginInstance::<Self>::instantiate),
                    connect_port: Some(::lv2_core::plugin::PluginInstance::<Self>::connect_port),
                    activate: Some(::lv2_core::plugin::PluginInstance::<Self>::activate),
                    run: Some(::lv2_core::plugin::PluginInstance::<Self>::run),
                    deactivate: Some(::lv2_core::plugin::PluginInstance::<Self>::deactivate),
                    cleanup: Some(::lv2_core::plugin::PluginInstance::<Self>::cleanup),
                    extension_data: Some(::lv2_core::plugin::PluginInstance::<Self>::extension_data)
                };
            }
        }
    }

    /// Create a matching arm for the plugin.
    ///
    /// The root function receives an index and has to return one plugin descriptor per index,
    /// or NULL. In this crate's implementation, this index is matched in a `match` statement and
    /// this method creates a match arm for this plugin.
    fn make_index_match_arm(&self, index: u32) -> impl ::quote::ToTokens {
        let plugin_type = &self.plugin_type;
        quote! {
            #index => &<#plugin_type as ::lv2_core::plugin::PluginInstanceDescriptor>::DESCRIPTOR,
        }
    }
}

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
    fn make_instance_descriptors<'a>(
        &'a self,
    ) -> impl Iterator<Item = impl ::quote::ToTokens> + 'a {
        self.descriptors
            .iter()
            .map(Lv2InstanceDescriptor::make_instance_descriptor_impl)
    }

    fn make_descriptor_function(&self) -> impl ::quote::ToTokens {
        let index_matchers = self
            .descriptors
            .iter()
            .enumerate()
            .map(|(i, desc)| desc.make_index_match_arm(i as u32));

        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn lv2_descriptor(index: u32) -> *const ::lv2_core::sys::LV2_Descriptor {
                match index {
                    #(#index_matchers)*
                    _ => ::std::ptr::null()
                }
            }
        }
    }
}

#[inline]
pub fn lv2_descriptors_impl(input: TokenStream) -> TokenStream {
    let list: Lv2InstanceDescriptorList = parse_macro_input!(input);
    let descriptors = list.make_instance_descriptors();
    let export_function = list.make_descriptor_function();

    (quote! {
        #(#descriptors)*
        #export_function
    })
    .into()
}
