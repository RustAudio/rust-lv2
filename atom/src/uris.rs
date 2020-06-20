//! Commonly used URIs from the lv2plug.in domain

#![allow(non_camel_case_types, non_snake_case)]

use urid::*;

pub struct AtomTransfer;

unsafe impl UriBound for AtomTransfer {
    const URI: &'static [u8] = sys::LV2_ATOM__atomTransfer;
}

pub struct BeatTime;

unsafe impl UriBound for BeatTime {
    const URI: &'static [u8] = sys::LV2_ATOM__beatTime;
}

pub struct BufferType;

unsafe impl UriBound for BufferType {
    const URI: &'static [u8] = sys::LV2_ATOM__bufferType;
}

pub struct ChildType;

unsafe impl UriBound for ChildType {
    const URI: &'static [u8] = sys::LV2_ATOM__childType;
}

pub struct EventTransfer;

unsafe impl UriBound for EventTransfer {
    const URI: &'static [u8] = sys::LV2_ATOM__eventTransfer;
}

pub struct FrameTime;

unsafe impl UriBound for FrameTime {
    const URI: &'static [u8] = sys::LV2_ATOM__frameTime;
}

pub struct Supports;

unsafe impl UriBound for Supports {
    const URI: &'static [u8] = sys::LV2_ATOM__supports;
}

pub struct TimeUnit;

unsafe impl UriBound for TimeUnit {
    const URI: &'static [u8] = sys::LV2_ATOM__timeUnit;
}

pub struct PatchAck;

unsafe impl UriBound for PatchAck {
    const URI: &'static [u8] = sys::LV2_PATCH__Ack;
}

pub struct PatchDelete;

unsafe impl UriBound for PatchDelete {
    const URI: &'static [u8] = sys::LV2_PATCH__Delete;
}

pub struct PatchCopy;

unsafe impl UriBound for PatchCopy {
    const URI: &'static [u8] = sys::LV2_PATCH__Copy;
}

pub struct PatchError;

unsafe impl UriBound for PatchError {
    const URI: &'static [u8] = sys::LV2_PATCH__Error;
}

pub struct PatchGet;

unsafe impl UriBound for PatchGet {
    const URI: &'static [u8] = sys::LV2_PATCH__Get;
}

pub struct PatchMessage;

unsafe impl UriBound for PatchMessage {
    const URI: &'static [u8] = sys::LV2_PATCH__Message;
}

pub struct PatchMove;

unsafe impl UriBound for PatchMove {
    const URI: &'static [u8] = sys::LV2_PATCH__Move;
}

pub struct PatchPatch;

unsafe impl UriBound for PatchPatch {
    const URI: &'static [u8] = sys::LV2_PATCH__Patch;
}

pub struct PatchPut;

unsafe impl UriBound for PatchPut {
    const URI: &'static [u8] = sys::LV2_PATCH__Put;
}

pub struct PatchPost;

unsafe impl UriBound for PatchPost {
    const URI: &'static [u8] = sys::LV2_PATCH__Post;
}

pub struct PatchRequest;

unsafe impl UriBound for PatchRequest {
    const URI: &'static [u8] = sys::LV2_PATCH__Request;
}

pub struct PatchResponse;

unsafe impl UriBound for PatchResponse {
    const URI: &'static [u8] = sys::LV2_PATCH__Response;
}

pub struct PatchSet;

unsafe impl UriBound for PatchSet {
    const URI: &'static [u8] = sys::LV2_PATCH__Set;
}

pub struct PatchAccept;

unsafe impl UriBound for PatchAccept {
    const URI: &'static [u8] = sys::LV2_PATCH__accept;
}

pub struct PatchAdd;

unsafe impl UriBound for PatchAdd {
    const URI: &'static [u8] = sys::LV2_PATCH__add;
}

pub struct PatchBody;

unsafe impl UriBound for PatchBody {
    const URI: &'static [u8] = sys::LV2_PATCH__body;
}

pub struct PatchContext;

unsafe impl UriBound for PatchContext {
    const URI: &'static [u8] = sys::LV2_PATCH__context;
}

pub struct PatchDestination;

unsafe impl UriBound for PatchDestination {
    const URI: &'static [u8] = sys::LV2_PATCH__destination;
}

pub struct PatchProperty;

unsafe impl UriBound for PatchProperty {
    const URI: &'static [u8] = sys::LV2_PATCH__property;
}

pub struct PatchReadable;

unsafe impl UriBound for PatchReadable {
    const URI: &'static [u8] = sys::LV2_PATCH__readable;
}

pub struct PatchRemove;

unsafe impl UriBound for PatchRemove {
    const URI: &'static [u8] = sys::LV2_PATCH__remove;
}

pub struct patch_request;

unsafe impl UriBound for patch_request {
    const URI: &'static [u8] = sys::LV2_PATCH__request;
}

pub struct PatchSubject;

unsafe impl UriBound for PatchSubject {
    const URI: &'static [u8] = sys::LV2_PATCH__subject;
}

pub struct PatchSequenceNumber;

unsafe impl UriBound for PatchSequenceNumber {
    const URI: &'static [u8] = sys::LV2_PATCH__sequenceNumber;
}

pub struct PatchValue;

unsafe impl UriBound for PatchValue {
    const URI: &'static [u8] = sys::LV2_PATCH__value;
}

pub struct PatchWildcard;

unsafe impl UriBound for PatchWildcard {
    const URI: &'static [u8] = sys::LV2_PATCH__wildcard;
}

pub struct PatchWritable;

unsafe impl UriBound for PatchWritable {
    const URI: &'static [u8] = sys::LV2_PATCH__writable;
}

#[derive(Clone, URIDCollection)]
pub struct PatchURIDCollection {
    pub ack: URID<PatchAck>,
    pub delete: URID<PatchDelete>,
    pub copy: URID<PatchCopy>,
    pub error: URID<PatchError>,
    pub get: URID<PatchGet>,
    pub message: URID<PatchMessage>,
    pub Move: URID<PatchMove>,
    pub patch: URID<PatchPatch>,
    pub post: URID<PatchPost>,
    pub put: URID<PatchPut>,
    pub Request: URID<PatchRequest>,
    pub response: URID<PatchResponse>,
    pub set: URID<PatchSet>,
    pub accept: URID<PatchAccept>,
    pub add: URID<PatchAdd>,
    pub body: URID<PatchBody>,
    pub context: URID<PatchContext>,
    pub destination: URID<PatchDestination>,
    pub property: URID<PatchProperty>,
    pub readable: URID<PatchReadable>,
    pub remove: URID<PatchRemove>,
    pub subject: URID<PatchSubject>,
    pub sequence_number: URID<PatchSequenceNumber>,
    pub value: URID<PatchValue>,
    pub wildcard: URID<PatchWildcard>,
    pub writable: URID<PatchWritable>,
}
