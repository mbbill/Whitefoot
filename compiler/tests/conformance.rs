#![forbid(unsafe_code)]

#[path = "support/link.rs"]
mod link_support;

mod conformance {
    mod adapter;
    mod corpus;
    mod json;
}
