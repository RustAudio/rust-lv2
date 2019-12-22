use crate::lib_name;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Path, Result, Token, Type};

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
    /// Implement the `PluginInstanceDescriptor` for the plugin.
    ///
    /// By implementing `PluginInstanceDescriptor`, two static objects are created: The URI of the
    /// plugin, stored as a string, and the descriptor, a struct with pointers to the plugin's
    /// basic functions; Like `instantiate` or `run`.
    pub fn make_instance_descriptor_impl(&self, crate_name: &Path) -> impl ::quote::ToTokens {
        let plugin_type = &self.plugin_type;
        quote! {
            unsafe impl #crate_name::plugin::PluginInstanceDescriptor for #plugin_type {
                const DESCRIPTOR: #crate_name::sys::LV2_Descriptor = #crate_name::sys::LV2_Descriptor {
                    URI: Self::URI.as_ptr() as *const u8 as *const ::std::os::raw::c_char,
                    instantiate: Some(#crate_name::plugin::PluginInstance::<Self>::instantiate),
                    connect_port: Some(#crate_name::plugin::PluginInstance::<Self>::connect_port),
                    activate: Some(#crate_name::plugin::PluginInstance::<Self>::activate),
                    run: Some(#crate_name::plugin::PluginInstance::<Self>::run),
                    deactivate: Some(#crate_name::plugin::PluginInstance::<Self>::deactivate),
                    cleanup: Some(#crate_name::plugin::PluginInstance::<Self>::cleanup),
                    extension_data: Some(#crate_name::plugin::PluginInstance::<Self>::extension_data)
                };
            }
        }
    }

    /// Create a matching arm for the plugin.
    ///
    /// The root function receives an index and has to return one plugin descriptor per index,
    /// or NULL. In this crate's implementation, this index is matched in a `match` statement and
    /// this method creates a match arm for this plugin.
    fn make_index_match_arm(&self, index: u32, crate_name: &Path) -> impl ::quote::ToTokens {
        let plugin_type = &self.plugin_type;
        quote! {
            #index => &<#plugin_type as #crate_name::plugin::PluginInstanceDescriptor>::DESCRIPTOR,
        }
    }
}

/// A container for instance descriptors.
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
    /// Implement `PluginInstanceDescriptor` for all plugin instances.
    fn make_instance_descriptor_impls<'a>(
        &'a self,
        crate_name: &'a Path,
    ) -> impl Iterator<Item = impl ::quote::ToTokens> + 'a {
        self.descriptors
            .iter()
            .map(move |d| d.make_instance_descriptor_impl(crate_name))
    }

    /// Create the `lv2_descriptor` function.
    ///
    /// This function tells the host of a library's plugin instances by returning one plugin
    /// instance per index.
    fn make_descriptor_function(&self, crate_name: &Path) -> impl ::quote::ToTokens {
        let index_matchers = self
            .descriptors
            .iter()
            .enumerate()
            .map(|(i, desc)| desc.make_index_match_arm(i as u32, crate_name));

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
            pub unsafe extern "C" fn lv2_descriptor(index: u32) -> *const #crate_name::sys::LV2_Descriptor {
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
    let crate_name = lib_name();
    let list: Lv2InstanceDescriptorList = parse_macro_input!(input);
    let descriptors = list.make_instance_descriptor_impls(&crate_name);
    let export_function = list.make_descriptor_function(&crate_name);

    (quote! {
        #(#descriptors)*
        #export_function
    })
    .into()
}
