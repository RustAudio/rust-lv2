//! Since this crate usese `bindgen` to create the C API bindings, you need to have clang installed on your machine.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(clippy::all)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    LV2_CORE__AllpassPlugin, LV2_CORE__AmplifierPlugin, LV2_CORE__AnalyserPlugin,
    LV2_CORE__AudioPort, LV2_CORE__BandpassPlugin, LV2_CORE__CVPort, LV2_CORE__ChorusPlugin,
    LV2_CORE__CombPlugin, LV2_CORE__CompressorPlugin, LV2_CORE__ConstantPlugin,
    LV2_CORE__ControlPort, LV2_CORE__ConverterPlugin, LV2_CORE__DelayPlugin,
    LV2_CORE__DistortionPlugin, LV2_CORE__DynamicsPlugin, LV2_CORE__EQPlugin,
    LV2_CORE__EnvelopePlugin, LV2_CORE__ExpanderPlugin, LV2_CORE__ExtensionData, LV2_CORE__Feature,
    LV2_CORE__FilterPlugin, LV2_CORE__FlangerPlugin, LV2_CORE__FunctionPlugin,
    LV2_CORE__GatePlugin, LV2_CORE__GeneratorPlugin, LV2_CORE__HighpassPlugin, LV2_CORE__InputPort,
    LV2_CORE__InstrumentPlugin, LV2_CORE__LimiterPlugin, LV2_CORE__LowpassPlugin,
    LV2_CORE__MixerPlugin, LV2_CORE__ModulatorPlugin, LV2_CORE__MultiEQPlugin,
    LV2_CORE__OscillatorPlugin, LV2_CORE__OutputPort, LV2_CORE__ParaEQPlugin,
    LV2_CORE__PhaserPlugin, LV2_CORE__PitchPlugin, LV2_CORE__Plugin, LV2_CORE__PluginBase,
    LV2_CORE__Point, LV2_CORE__Port, LV2_CORE__PortProperty, LV2_CORE__Resource,
    LV2_CORE__ReverbPlugin, LV2_CORE__ScalePoint, LV2_CORE__SimulatorPlugin,
    LV2_CORE__SpatialPlugin, LV2_CORE__Specification, LV2_CORE__SpectralPlugin,
    LV2_CORE__UtilityPlugin, LV2_CORE__WaveshaperPlugin, LV2_CORE__appliesTo, LV2_CORE__binary,
    LV2_CORE__connectionOptional, LV2_CORE__control, LV2_CORE__default, LV2_CORE__designation,
    LV2_CORE__documentation, LV2_CORE__enumeration, LV2_CORE__extensionData,
    LV2_CORE__freeWheeling, LV2_CORE__hardRTCapable, LV2_CORE__inPlaceBroken, LV2_CORE__index,
    LV2_CORE__integer, LV2_CORE__isLive, LV2_CORE__latency, LV2_CORE__maximum,
    LV2_CORE__microVersion, LV2_CORE__minimum, LV2_CORE__minorVersion, LV2_CORE__name,
    LV2_CORE__optionalFeature, LV2_CORE__port, LV2_CORE__portProperty, LV2_CORE__project,
    LV2_CORE__prototype, LV2_CORE__reportsLatency, LV2_CORE__requiredFeature, LV2_CORE__sampleRate,
    LV2_CORE__scalePoint, LV2_CORE__symbol, LV2_CORE__toggled, LV2_Descriptor,
    LV2_Descriptor_Function, LV2_Feature, LV2_Handle, LV2_Lib_Descriptor,
    LV2_Lib_Descriptor_Function, LV2_Lib_Handle, _LV2_Descriptor, _LV2_Feature, LV2_CORE_PREFIX,
    LV2_CORE_URI,
};
