use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Result, Token, Type};

/// An instance descriptor that should be exported.
///
/// The instance descriptor is defined by the plugin type.
struct Lv2UIInstanceDescriptor {
    plugin_type: Type,
}

impl Parse for Lv2UIInstanceDescriptor {
    fn parse(input: ParseStream) -> Result<Self> {
        let plugin_type = input.parse()?;
        Ok(Lv2UIInstanceDescriptor { plugin_type })
    }
}

impl Lv2UIInstanceDescriptor {
    /// Implement the `PluginUIInstanceDescriptor` for the plugin.
    ///
    /// By implementing `PluginUIInstanceDescriptor`, two static objects are created: The URI of the
    /// plugin, stored as a string, and the descriptor, a struct with pointers to the plugin's
    /// basic functions; Like `instantiate` or `run`.
    pub fn make_instance_descriptor_impl(&self) -> impl ::quote::ToTokens {
        let plugin_type = &self.plugin_type;
        quote! {
            unsafe impl PluginUIInstanceDescriptor for #plugin_type {
                const DESCRIPTOR: LV2UI_Descriptor = LV2UI_Descriptor {
                    URI: Self::URI.as_ptr() as *const u8 as *const ::std::os::raw::c_char,
                    instantiate: Some(PluginUIInstance::<Self>::instantiate),
                    cleanup: Some(PluginUIInstance::<Self>::cleanup),
                    port_event: Some(PluginUIInstance::<Self>::port_event),
                    extension_data: Some(PluginUIInstance::<Self>::extension_data)
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
            #index => &<#plugin_type as PluginUIInstanceDescriptor>::DESCRIPTOR,
        }
    }
}

/// A collection for instance descriptors.
///
/// The contained instance descriptors are used to create the export function `lv2_ui_descriptor` that
/// tells the host of a library's plugins.
struct Lv2UIInstanceDescriptorList {
    descriptors: Punctuated<Lv2UIInstanceDescriptor, Token![,]>,
}

impl Parse for Lv2UIInstanceDescriptorList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            descriptors: Punctuated::parse_terminated(input)?,
        })
    }
}

impl Lv2UIInstanceDescriptorList {
    /// Implement `PluginUIInstanceDescriptor` for all plugin instances.
    fn make_instance_descriptor_impls<'a>(
        &'a self,
    ) -> impl Iterator<Item = impl ::quote::ToTokens> + 'a {
        self.descriptors
            .iter()
            .map(Lv2UIInstanceDescriptor::make_instance_descriptor_impl)
    }

    /// Create the `lv2_ui_descriptor` function.
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
            pub unsafe extern "C" fn lv2ui_descriptor(index: u32) -> *const LV2UI_Descriptor {
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
pub fn lv2_ui_descriptors_impl(input: TokenStream) -> TokenStream {
    let list: Lv2UIInstanceDescriptorList = parse_macro_input!(input);
    let descriptors = list.make_instance_descriptor_impls();
    let export_function = list.make_descriptor_function();

    (quote! {
        #(#descriptors)*
        #export_function
    })
    .into()
}
